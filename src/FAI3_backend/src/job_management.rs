use crate::types::Job;
use crate::{JOBS, NEXT_JOB_ID};

fn create_job(model_id: u128) -> u128 {
    let id = NEXT_JOB_ID.with(|id| {
        let id = id.get();
        id + 1
    });
    let job = Job {
        id,
        model_id,
        status: "Pending".to_string(),
        timestamp: ic_cdk::api::time(),
    };
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        jobs.insert(id, job.clone());
    });

    job.id
}

fn get_job(job_id: u128) -> Option<Job> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        jobs.get(&job_id).cloned()
    })
}

fn get_jobs() -> Vec<Job> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        jobs.values().cloned().collect()
    })
}

fn update_job_status(job_id: u128, status: &str) {
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = status.to_string();
        }
    });
}

fn get_job_status(job_id: u128) -> Option<String> {
    JOBS.with(|jobs| {
        let jobs = jobs.borrow();
        if let Some(job) = jobs.get(&job_id) {
            Some(job.status.clone())
        } else {
            None
        }
    })
}

fn delete_job(job_id: u128) {
    JOBS.with(|jobs| {
        let mut jobs = jobs.borrow_mut();
        jobs.remove(&job_id);
    });
}
