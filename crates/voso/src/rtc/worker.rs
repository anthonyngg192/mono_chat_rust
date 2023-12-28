use mediasoup::{worker::Worker, worker::WorkerSettings, worker_manager::WorkerManager};
use once_cell::sync::OnceCell;

use crate::util::variables::{RTC_MAX_PORT, RTC_MIN_PORT};
pub static WORKER_POOL: OnceCell<WorkerPool> = OnceCell::new();

pub fn get_worker_pool() -> &'static WorkerPool {
    WORKER_POOL
        .get()
        .expect("Mediasoup worker pool not initialized")
}

#[derive(Debug)]
pub struct WorkerPool {
    worker: Worker,
}

impl WorkerPool {
    pub async fn new() -> Self {
        let manager = WorkerManager::new();

        let mut settings = WorkerSettings::default();

        settings.rtc_ports_range = (*RTC_MIN_PORT)..=(*RTC_MAX_PORT);

        let worker = manager.create_worker(settings).await.unwrap();

        debug!("Initialized worker pool");
        WorkerPool { worker }
    }

    pub fn get_worker(&self) -> &Worker {
        &self.worker
    }
}
