use std::collections::{HashMap, HashSet};

use crate::models::{Channel, Member, Server, User};

/// Enumeration representing some change in subscriptions
pub enum SubscriptionStateChange {
    /// No change
    None,
    /// Clear all subscriptions
    Reset,
    /// Append or remove subscriptions
    Change {
        add: Vec<String>,
        remove: Vec<String>,
    },
}

#[derive(Debug, Default)]
pub struct Cache {
    pub user_id: String,

    pub users: HashMap<String, User>,
    pub channels: HashMap<String, Channel>,
    pub members: HashMap<String, Member>,
    pub servers: HashMap<String, Server>,
}

pub struct State {
    pub cache: Cache,

    pub private_topic: String,
    subscribed: HashSet<String>,
    state: SubscriptionStateChange,
}

impl State {
    pub fn from(user: User) -> State {
        let mut subscribed = HashSet::new();

        let private_topic = format!("{}!", user.id);

        subscribed.insert(private_topic.clone());
        subscribed.insert(user.id.clone());

        let mut cache = Cache {
            user_id: user.id.clone(),
            ..Default::default()
        };

        cache.users.insert(user.id.clone(), user);

        State {
            cache,
            private_topic,
            subscribed,
            state: SubscriptionStateChange::Reset,
        }
    }

    pub fn apply_state(&mut self) -> SubscriptionStateChange {
        let state = std::mem::replace(&mut self.state, SubscriptionStateChange::None);

        if let SubscriptionStateChange::Change { add, remove } = &state {
            for id in add {
                self.subscribed.insert(id.clone());
            }

            for id in remove {
                self.subscribed.remove(id);
            }
        }
        state
    }

    pub fn clone_user(&self) -> User {
        self.cache.users.get(&self.cache.user_id).unwrap().clone()
    }

    pub fn iter_subscriptions(&self) -> std::collections::hash_set::Iter<'_, std::string::String> {
        self.subscribed.iter()
    }

    pub fn reset_state(&mut self) {
        self.state = SubscriptionStateChange::Reset;
        self.subscribed.clear();
    }

    pub fn insert_subscription(&mut self, subscription: String) {
        if self.subscribed.contains(&subscription) {
            return;
        }

        match &mut self.state {
            SubscriptionStateChange::None => {
                self.state = SubscriptionStateChange::Change {
                    add: vec![subscription.clone()],
                    remove: vec![],
                };
            }
            SubscriptionStateChange::Change { add, .. } => {
                add.push(subscription.clone());
            }
            SubscriptionStateChange::Reset => {}
        }

        self.subscribed.insert(subscription);
    }

    pub fn remove_subscription(&mut self, subscription: &str) {
        if !self.subscribed.contains(&subscription.to_string()) {
            return;
        }

        match &mut self.state {
            SubscriptionStateChange::None => {
                self.state = SubscriptionStateChange::Change {
                    add: vec![],
                    remove: vec![subscription.to_string()],
                };
            }
            SubscriptionStateChange::Change { remove, .. } => {
                remove.push(subscription.to_string());
            }
            SubscriptionStateChange::Reset => panic!("Should not remove during a reset!"),
        }

        self.subscribed.remove(subscription);
    }
}
