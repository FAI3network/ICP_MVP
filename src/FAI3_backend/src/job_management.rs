use candid::Principal;
use crate::types::{Job, JobType, JobProgress};
use crate::{only_admin, JOBS, NEXT_JOB_ID, LAST_PROCESSED_JOB_ID};
use std::cell::RefCell;
use std::collections::HashSet;

pub const JOB_STATUS_PENDING: &str = "Pending";
pub const JOB_STATUS_IN_PROGRESS: &str = "In Progress";
pub const JOB_STATUS_COMPLETED: &str = "Completed";
pub const JOB_STATUS_FAILED: &str = "Failed";
pub const JOB_STATUS_STOPPED: &str = "Stopped";
pub const JOB_STATUS_PAUSED: &str = "Paused";

const QUEUE_CYCLE_THRESHOLD: u64 = 40_000_000_000;

thread_local! {
    static QUEUE_BUSY: RefCell<bool> = RefCell::new(false);
    static STOP_JOB: RefCell<HashSet<u128>> = RefCell::new(HashSet::new());
}

#[ic_cdk::update]
pub fn create_job(model_id: u128) -> u128 {
    only_admin();
    let id = NEXT_JOB_ID.with(|id| {
        let current_id = *id.borrow().get();

        id.borrow_mut().set(current_id + 1).unwrap();

        current_id
    });

    let owner_id = ic_cdk::caller();

    let job = Job {
        id,
        model_id,
        owner: owner_id,
        status: JOB_STATUS_PENDING.to_string(),
        timestamp: ic_cdk::api::time(),
        job_type: JobType::Unassigned,
        status_detail: None,
        progress: Default::default(),
    };
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        jobs.insert(id, job.clone());
    });

    job.id
}

pub fn create_job_with_job_type(model_id: u128, job_type: JobType, max_queries: usize) -> u128 {
    let id = NEXT_JOB_ID.with(|id| {
        let current_id = *id.borrow().get();

        id.borrow_mut().set(current_id + 1).unwrap();

        current_id
    });

    let owner_id = ic_cdk::caller();

    let job = Job {
        id,
        model_id,
        owner: owner_id,
        status: JOB_STATUS_PENDING.to_string(),
        timestamp: ic_cdk::api::time(),
        job_type,
        status_detail: None,
        progress: JobProgress {
            completed: 0,
            target: max_queries,
            invalid_responses: 0,
            call_errors: 0,
        },
    };
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        jobs.insert(id, job.clone());
    });

    job.id
}

#[ic_cdk::query]
pub fn get_job(job_id: u128) -> Option<Job> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        jobs.get(&job_id).map(|job| job.clone())
    })
}

#[ic_cdk::query]
pub fn get_jobs() -> Vec<Job> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        jobs.values().map(|job| job.clone()).collect()
    })
}

#[ic_cdk::update]
pub fn update_job_status(job_id: u128, status: String, model_id: u128) {
    only_admin();
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        if let Some(job) = jobs.get(&job_id) {
            if job.model_id != model_id {
                panic!("Model ID mismatch");
            }

            let mut updated_job = job.clone();
            updated_job.status = status;
            jobs.insert(job_id, updated_job);
        }
    });
}

pub fn job_fail(job_id: u128, model_id: u128) {
    update_job_status(job_id, JOB_STATUS_FAILED.to_string(), model_id);
}

pub fn job_complete(job_id: u128, model_id: u128) {
    update_job_status(job_id, JOB_STATUS_COMPLETED.to_string(), model_id);
}

pub fn job_in_progress(job_id: u128, model_id: u128) {
    update_job_status(job_id, JOB_STATUS_IN_PROGRESS.to_string(), model_id);
}

#[ic_cdk::query]
pub fn get_job_status(job_id: u128) -> Option<String> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        if let Some(job) = jobs.get(&job_id) {
            Some(job.status.clone())
        } else {
            None
        }
    })
}

#[ic_cdk::query]
pub fn check_job_stopped(job_id: u128) -> bool {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        if let Some(job) = jobs.get(&job_id) {
            job.status == JOB_STATUS_STOPPED.to_string()
        } else {
            false
        }
    })
}

#[ic_cdk::update]
pub fn stop_job(job_id: u128) {
    only_admin();
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        if let Some(job) = jobs.get(&job_id) {
            let mut updated_job = job.clone();
            updated_job.status = JOB_STATUS_STOPPED.to_string();
            jobs.insert(job_id, updated_job);
        }

        // This is required to stop projects running on the queue
        STOP_JOB.with(|stop_jobs| {
            stop_jobs.borrow_mut().insert(job_id);
        });
    });
}

#[ic_cdk::query]
pub fn get_latest_job() -> Option<Job> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        let mut latest_job: Option<Job> = None;
        for job in jobs.values() {
            if latest_job.is_none() || job.timestamp > latest_job.as_ref().unwrap().timestamp {
                latest_job = Some(job.clone());
            }
        }
        latest_job
    })
}

#[ic_cdk::query]
pub fn get_job_by_model_id(model_id: u128) -> Option<Job> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        for job in jobs.values() {
            if job.model_id == model_id {
                return Some(job.clone());
            }
        }
        None
    })
}

#[ic_cdk::query]
pub fn get_job_by_owner(owner_id: Principal) -> Vec<Job> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        let mut result = Vec::new();
        for job in jobs.values() {
            if job.owner == owner_id {
                result.push(job.clone());
            }
        }
        result
    })
}

// Internal job methods (do not require admin, which fails on timers, and are not exposed)
pub fn internal_update_job_status(job_id: u128, status: String,
                                  model_id: u128, status_detail: Option<String>,
                                  completed: Option<usize>,
                                  invalid_responses: Option<usize>,
                                  call_errors: Option<usize>) {
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        if let Some(job) = jobs.get(&job_id) {
            if job.model_id != model_id {
                panic!("Model ID mismatch");
            }

            let mut updated_job = job.clone();
            updated_job.status = status;

            if let Some(sts_detail) = status_detail {
                updated_job.status_detail = Some(sts_detail);
            }

            // Progress information
            if let Some(_completed) = completed {
                updated_job.progress.completed = _completed;

                ic_cdk::println!("Saving progress for job {}: {}/{}", job.id, _completed, job.progress.target);
            }
            if let Some(_invalid_responses) = invalid_responses {
                updated_job.progress.invalid_responses = _invalid_responses;

            }
            if let Some(_call_errors) = call_errors {
                updated_job.progress.call_errors = _call_errors;

            }
            
            jobs.insert(job_id, updated_job);
        }
    });
}

pub fn internal_job_fail(job_id: u128, model_id: u128, error_detail: Option<String>) {
    internal_update_job_status(job_id, JOB_STATUS_FAILED.to_string(), model_id, error_detail, None, None, None);
}

pub fn internal_job_complete(job_id: u128, model_id: u128) {
    internal_update_job_status(job_id, JOB_STATUS_COMPLETED.to_string(), model_id, None, None, None, None);
}

pub fn internal_job_in_progress(job_id: u128, model_id: u128, completed: usize, invalid_responses: usize, call_errors: usize) {
    internal_update_job_status(job_id, JOB_STATUS_IN_PROGRESS.to_string(), model_id, None, Some(completed), Some(invalid_responses), Some(call_errors));
}

pub fn internal_job_stop(job_id: u128, model_id: u128) {
    internal_update_job_status(job_id, JOB_STATUS_STOPPED.to_string(), model_id, None, None, None, None);

    // Remove it from the queue
    STOP_JOB.with(|stop_jobs| {
        stop_jobs.borrow_mut().remove(&job_id);
    });
}

pub fn job_should_be_stopped(job_id: u128) -> bool {
    return STOP_JOB.with(|stop_jobs| {
        let stop_jobs = stop_jobs.borrow();
        return stop_jobs.contains(&job_id);
    });
}

// JOB QUEUE

/// Inits job queue
#[ic_cdk::update]
pub fn restart_job_queue() {
    ic_cdk::println!("Restarting job queue");
    bootstrap_job_queue();
}

pub fn bootstrap_job_queue() {
    ic_cdk_timers::set_timer(
        core::time::Duration::from_secs(1),
        || {
            // Check if the queue is already processing
            let already_busy = QUEUE_BUSY.with(|busy| {
                let is_busy = *busy.borrow();
                if !is_busy {
                    *busy.borrow_mut() = true;
                    false
                } else {
                    true
                }
            });

            if !already_busy {
                ic_cdk::spawn(async {

                    crate::job_management::process_job_queue().await;

                    // Release the lock when done
                    QUEUE_BUSY.with(|busy| {
                        *busy.borrow_mut() = false;
                    });
                });

            } else {
                ic_cdk::println!("Queue already processing, skipping this run");
            }
             
        }
    );
}

pub fn get_next_unfinished_job() -> Option<Job> {
    // get_jobs could be used but it wouldn't be efficient
    return NEXT_JOB_ID.with(|id| {
        let next_job_id = *id.borrow().get();

        let mut last_processed_id = 0;
        
        LAST_PROCESSED_JOB_ID.with(| id | {
             last_processed_id = *id.borrow().get();
        });

        let mut next_job_to_process = last_processed_id + 1;

        while next_job_to_process < next_job_id {
            let job = get_job(next_job_to_process);

            match job {
                Some(_job) => {
                    if _job.status == JOB_STATUS_PENDING
                        || _job.status == JOB_STATUS_IN_PROGRESS {
                            // Job status stopped should be handled 
                            return Some(_job);
                        }
                },
                None => (),
            }

            next_job_to_process += 1;
        }

        return Option::<Job>::None;
    });
}

pub fn set_last_processed_job_id(job_id: u128) {
    return LAST_PROCESSED_JOB_ID.with(| id | {
        let current_id = *id.borrow().get();

        if current_id > job_id {
            ic_cdk::println!("Trying to set a job_id = {} when current last_processed_job_id = {}", job_id, current_id);
            return;
        }
        id.borrow_mut().set(job_id).unwrap();
    });
}


/// Processes the job queue
///
/// The responsability for updating jobs and memory elements
/// are in each module. The queue just calls the modules
/// And updates the last_processed_job memory number.
pub async fn process_job_queue() {
    // If balance is low, then we stop the queue
    let cycles: u64 = ic_cdk::api::canister_balance();
    if cycles < QUEUE_CYCLE_THRESHOLD {
        ic_cdk::println!("Cycle balance too low, stopping execution to avoid canister deletion.");
        return;
    }

    let job = get_next_unfinished_job();

    if job == None {
        ic_cdk::println!("Queue: no work to do. Stopping queue.");
        return;
    }

    let job = job.unwrap();

    if job.status == JOB_STATUS_PENDING {
        ic_cdk::println!("Starting job {}", job.id);
        internal_job_in_progress(job.id, job.model_id, 0, 0, 0);
    }

    if job.status == JOB_STATUS_PAUSED {
        ic_cdk::println!("Job with id = {} has status = PAUSED. Set it to pending again to restore the queue processing", job.id);
        return;
    }

    ic_cdk::println!("Processing job {}", job.id);

    let is_evaluation_finished = match job.job_type {
        JobType::LLMFairness { model_evaluation_id } => {
            crate::llm_fairness::llm_metrics_process_next_query(job.model_id, model_evaluation_id, &job).await
        },
        JobType::AverageFairness { ref job_dependencies } => {
            crate::llm_fairness::process_average_llm_metrics_from_job(&job, job_dependencies.clone())
        },
        _ => {
            ic_cdk::println!("Job type not supported yet. Ignoring it.");
            Ok(true)
        },
    };

    match is_evaluation_finished {
        Ok(job_finished) => {
            if job_finished {
                set_last_processed_job_id(job.id);
                ic_cdk::println!("Next job to be processed: {}", job.id + 1);
            }

            // If there is still work to do, we keep processing the queue
            bootstrap_job_queue();
        },
        Err(error_string) => {
            ic_cdk::eprintln!("An grave error happened when processing this job. Stopping queue. Error: {}", &error_string);
        }
    }
}
