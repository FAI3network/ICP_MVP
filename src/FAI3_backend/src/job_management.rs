use candid::Principal;

use crate::types::Job;
use crate::{JOBS, NEXT_JOB_ID};

#[ic_cdk::update]
pub fn create_job(model_id: u128) -> u128 {
    let id = NEXT_JOB_ID.with(|id| {
        let current_id = *id.borrow().get();

        id.borrow_mut().set(current_id + 1).unwrap();

        current_id
    });

    let owner_id = ic_cdk::caller();

    let job = Job {
        id: id,
        model_id,
        owner: owner_id,
        status: "Pending".to_string(),
        timestamp: ic_cdk::api::time(),
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

#[ic_cdk::update]
pub fn job_fail(job_id: u128, model_id: u128) {
    update_job_status(job_id, "Failed".to_string(), model_id);
}

#[ic_cdk::update]
pub fn job_complete(job_id: u128, model_id: u128) {
    update_job_status(job_id, "Completed".to_string(), model_id);
}

#[ic_cdk::update]
pub fn job_in_progress(job_id: u128, model_id: u128) {
    update_job_status(job_id, "In Progress".to_string(), model_id);
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

#[ic_cdk::update]
pub fn delete_job(job_id: u128) {
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        jobs.remove(&job_id);
    });
}

#[ic_cdk::query]
pub fn check_job_stopped(job_id: u128) -> bool {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        if let Some(job) = jobs.get(&job_id) {
            job.status == "Stopped"
        } else {
            false
        }
    })
}

#[ic_cdk::update]
pub fn stop_job(job_id: u128) {
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        if let Some(job) = jobs.get(&job_id) {
            let mut updated_job = job.clone();
            updated_job.status = "Stopped".to_string();
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
