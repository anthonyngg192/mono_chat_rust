use std::{collections::HashMap, time::Duration};

use deadqueue::limited::Queue;
use once_cell::sync::Lazy;

use crate::{database::Database, models::channel::PartialChannel};

use super::DelayedTask;

struct Data {
    channel: String,
    id: String,
    is_dm: bool,
}

#[derive(Debug)]
struct Task {
    id: String,
    is_dm: bool,
}

static Q: Lazy<Queue<Data>> = Lazy::new(|| Queue::new(10_000));

pub async fn queue(channel: String, id: String, is_dm: bool) {
    Q.try_push(Data { channel, id, is_dm }).ok();
    info!("Queue is using {} slots from {}.", Q.len(), Q.capacity());
}

pub async fn worker(db: Database) {
    let mut tasks = HashMap::<String, DelayedTask<Task>>::new();
    let mut keys = vec![];

    loop {
        for (key, task) in &tasks {
            if task.should_run() {
                keys.push(key.clone());
            }
        }

        for key in &keys {
            if let Some(task) = tasks.remove(key) {
                let Task { id, is_dm, .. } = task.data;

                let mut channel = PartialChannel {
                    last_message_id: Some(id.to_string()),
                    ..Default::default()
                };

                if is_dm {
                    channel.active = Some(true)
                }

                match db.update_channel(key, &channel, vec![]).await {
                    Ok(_) => info!("Updated last_message_id for {key} to {id}."),
                    Err(err) => error!("Failed to update last_message_id with {err:?}!"),
                }
            }
        }

        keys.clear();

        while let Some(Data { channel, id, is_dm }) = Q.try_pop() {
            if let Some(task) = tasks.get_mut(&channel) {
                task.data.id = id;
                task.delay();
            } else {
                tasks.insert(channel, DelayedTask::new(Task { id, is_dm }));
            }
        }

        async_std::task::sleep(Duration::from_secs(1)).await;
    }
}
