use std::sync::mpsc::Sender;

use chrono::prelude::*;
use chrono::Duration;

#[derive(Clone)]
pub struct Progress {
    started_at: DateTime<Local>,
    completed: i64,
    total: i64,
    sender: Sender<Progress>,
}

impl Progress {
    pub fn new(sender: Sender<Progress>, total: usize) -> Self {
        let total = total as i64;
        Progress {
            started_at: Local::now(),
            completed: 0,
            total,
            sender,
        }
    }

    pub fn increment(&mut self) {
        self.completed += 1;
        self.sender.send(self.clone()).unwrap();
    }

    pub fn status_msg(&self) -> String {
        let elapsed = self.elapsed();
        let estimate = Duration::seconds(self.total * elapsed.num_seconds() / self.completed);
        let percentage = self.completed * 100 / self.total;
        format!(
            "{:>3}% completed (ETR: {} seconds)",
            percentage,
            estimate.num_seconds() - elapsed.num_seconds()
        ).into()
    }

    pub fn elapsed(&self) -> Duration {
        Local::now().signed_duration_since(self.started_at)
    }
}
