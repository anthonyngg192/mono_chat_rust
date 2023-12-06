use async_std::task;
use std::time::Instant;

const WORKER_COUNT: usize = 5;

use crate::Database;
pub mod ack;
pub mod last_message_id;
pub mod process_embeds;
pub mod web_push;

pub struct DelayedTask<T> {
    pub data: T,
    pub last_updated: Instant,
    pub first_seen: Instant,
}

pub async fn start_workers(db: Database, authifier_db: authifier::Database) {
    for _ in 0..WORKER_COUNT {
        task::spawn(ack::worker(db.clone()));
        task::spawn(last_message_id::worker(db.clone()));
        task::spawn(process_embeds::worker(db.clone()));
        task::spawn(web_push::worker(authifier_db.clone()));
    }
}

static EXPIRE_CONSTANT: u64 = 30;

static SAVE_CONSTANT: u64 = 5;

impl<T> DelayedTask<T> {
    pub fn new(data: T) -> Self {
        DelayedTask {
            data,
            last_updated: Instant::now(),
            first_seen: Instant::now(),
        }
    }

    pub fn delay(&mut self) {
        self.last_updated = Instant::now();
    }

    pub fn should_run(&self) -> bool {
        self.first_seen.elapsed().as_secs() > EXPIRE_CONSTANT
            || self.last_updated.elapsed().as_secs() > SAVE_CONSTANT
    }
}
