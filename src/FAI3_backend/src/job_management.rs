use candid::Principal;

use crate::types::{Job, JobType};
use crate::{only_admin, JOBS, NEXT_JOB_ID, LAST_PROCESSED_JOB_ID};

pub const JOB_STATUS_PENDING: &str = "Pending";
pub const JOB_STATUS_IN_PROGRESS: &str = "In Progress";
pub const JOB_STATUS_COMPLETED: &str = "Completed";
pub const JOB_STATUS_FAILED: &str = "Failed";
pub const JOB_STATUS_STOPPED: &str = "Stopped";

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
    };
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        jobs.insert(id, job.clone());
    });

    job.id
}

pub fn create_job_with_job_type(model_id: u128, job_type: JobType) -> u128 {
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
pub fn internal_update_job_status(job_id: u128, status: String, model_id: u128, status_detail: Option<String>) {
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
            
            jobs.insert(job_id, updated_job);
        }
    });
}

pub fn internal_job_fail(job_id: u128, model_id: u128, error_detail: Option<String>) {
    internal_update_job_status(job_id, JOB_STATUS_FAILED.to_string(), model_id, error_detail);
}

pub fn internal_job_complete(job_id: u128, model_id: u128) {
    internal_update_job_status(job_id, JOB_STATUS_COMPLETED.to_string(), model_id, None);
}

pub fn internal_job_in_progress(job_id: u128, model_id: u128) {
    internal_update_job_status(job_id, JOB_STATUS_IN_PROGRESS.to_string(), model_id, None);
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
        || ic_cdk::spawn(crate::job_management::process_job_queue()));
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

pub fn increment_last_processed_job_id() -> u128 {
    return LAST_PROCESSED_JOB_ID.with(| id | {
        let current_id = *id.borrow().get();

        id.borrow_mut().set(current_id + 1).unwrap();

        current_id + 1
    });
}


/// Processes the job queue
///
/// The responsability for updating jobs and memory elements
/// are in each module. The queue just calls the modules
/// And updates the last_processed_job memory number.
pub async fn process_job_queue() {

    let job = get_next_unfinished_job();

    if job == None {
        ic_cdk::println!("Queue: no work to do. Stopping queue.");
        return;
    }

    let job = job.unwrap();

    if job.status == JOB_STATUS_PENDING {
        ic_cdk::println!("Starting job {}", job.id);
        internal_job_in_progress(job.id, job.model_id);
    }

    let is_evaluation_finished = match job.job_type {
        JobType::LLMFairness { model_evaluation_id } => {
            crate::llm_fairness::llm_metrics_process_next_query(job.model_id, model_evaluation_id, &job).await
        },
        JobType::AverageFairness { ref job_dependencies } => {
            crate::llm_fairness::process_average_llm_metrics_from_job(&job, job_dependencies.clone())
        },
        _ => {
            ic_cdk::println!("Job type not supported yet.");
            Ok(true)
        },
    };

    match is_evaluation_finished {
        Ok(job_finished) => {
            if job_finished {
                let last_processed_job_id = increment_last_processed_job_id();
                ic_cdk::println!("Next job to be processed: {}", last_processed_job_id + 1);
            }

            // If there is still work to do, we keep processing the queue
            bootstrap_job_queue();
        },
        Err(error_string) => {
            ic_cdk::eprintln!("An grave error happened when processing this job. Stopping queue. Error: {}", &error_string);
        }
    }
}
