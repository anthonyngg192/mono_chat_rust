use std::{collections::HashMap, time::Duration};

use crate::database::Database;

use super::DelayedTask;
use deadqueue::limited::Queue;
use once_cell::sync::Lazy;

#[derive(Debug, Eq, PartialEq)]
pub enum AckEvent {
    AddMention { ids: Vec<String> },

    AckMessage { id: String },
}

struct Data {
    channel: String,
    user: String,
    event: AckEvent,
}

#[derive(Debug)]
pub struct Task {
    event: AckEvent,
}

static Q: Lazy<Queue<Data>> = Lazy::new(|| Queue::new(10_000));

pub async fn queue(channel: String, user: String, event: AckEvent) {
    Q.try_push(Data {
        channel,
        user,
        event,
    })
    .ok();

    info!("Queue is using {} slot from {}.", Q.len(), Q.capacity());
}

pub async fn worker(db: Database) {
    let mut tasks = HashMap::<(String, String), DelayedTask<Task>>::new();

    let mut keys = vec![];

    loop {
        for (key, task) in &tasks {
            if task.should_run() {
                keys.push(key.clone());
            }
        }
        for key in &keys {
            if let Some(task) = tasks.remove(key) {
                let Task { event } = task.data;
                let (user, channel) = key;

                if let Err(err) = match &event {
                    AckEvent::AckMessage { id } => db.acknowledge_message(channel, user, id).await,
                    AckEvent::AddMention { ids } => {
                        db.add_mention_to_unread(channel, user, ids).await
                    }
                } {
                    error!("{err:?} for {event:?}. ({user}, {channel})");
                } else {
                    info!("User {user} ack in {channel} with {event:?}");
                }
            }
        }
        keys.clear();

        while let Some(Data {
            channel,
            user,
            mut event,
        }) = Q.try_pop()
        {
            let key = (user, channel);
            if let Some(task) = tasks.get_mut(&key) {
                task.delay();

                match &mut event {
                    AckEvent::AddMention { ids } => {
                        if let AckEvent::AddMention { ids: existing } = &mut task.data.event {
                            existing.append(ids);
                        } else {
                            task.data.event = event;
                        }
                    }
                    AckEvent::AckMessage { .. } => {
                        task.data.event = event;
                    }
                }
            } else {
                tasks.insert(key, DelayedTask::new(Task { event }));
            }
        }

        async_std::task::sleep(Duration::from_secs(1)).await
    }
}
