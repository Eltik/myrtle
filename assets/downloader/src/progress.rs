use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct ProgressTracker {
    multi: MultiProgress,
    total: ProgressBar,
}

impl ProgressTracker {
    pub fn new(total_files: u64) -> Self {
        let multi = MultiProgress::new();
        let style =
            ProgressStyle::with_template("{prefix} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("=> ");

        let total = multi.add(ProgressBar::new(total_files));
        total.set_style(style);
        total.set_prefix("Downloading");

        Self { multi, total }
    }

    pub fn inc(&self) {
        self.total.inc(1);
    }
    pub fn finish(&self) {
        self.total.finish_with_message("done");
    }
    pub fn println(&self, msg: &str) {
        self.multi.println(msg).ok();
    }
}
