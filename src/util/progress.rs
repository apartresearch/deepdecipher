use std::{
    io::{self, Write},
    time::Instant,
};

pub struct Progress {
    start: Instant,
    progress: u64,
    total: u64,
    message: String,
}

impl Progress {
    pub fn start(total: u64, message: &str) -> Self {
        Self {
            start: Instant::now(),
            progress: 0,
            total,
            message: message.to_string(),
        }
    }

    pub fn increment(&mut self) {
        self.progress += 1;
        assert!(self.progress <= self.total);
    }

    fn message(&self) -> String {
        let percent = (self.progress as f64 / self.total as f64) * 100.0;
        let elapsed = self.start.elapsed();
        let time_remaining = if self.progress > 0 {
            let time_per_element = elapsed.div_f64(self.progress as f64);
            format!(
                "{:?}s",
                time_per_element
                    .mul_f64((self.total - self.progress) as f64)
                    .as_secs()
            )
        } else {
            "NaN".to_string()
        };
        format!(
            "{}: {:.2}% ({}/{}) Est. time remaining: {:?}",
            self.message, percent, self.progress, self.total, time_remaining
        )
    }

    pub fn print(&self) {
        print!("\r{}", self.message());
        io::stdout().flush().unwrap();
    }
}
