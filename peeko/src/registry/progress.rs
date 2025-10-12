#[cfg(feature = "progress")]
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

// Progress trait abstraction
pub trait ProgressTracker: Send + Sync {
    fn start_download(&self, digest: &str, total_bytes: u64);
    fn update(&self, digest: &str, bytes: u64);
    fn finish(&self, digest: &str);
}

// No-op implementation
pub struct NoopProgress;

impl ProgressTracker for NoopProgress {
    fn start_download(&self, _digest: &str, _total_bytes: u64) {}
    fn update(&self, _digest: &str, _bytes: u64) {}
    fn finish(&self, _digest: &str) {}
}

// Indicatif implementation (only when feature enabled)
#[cfg(feature = "progress")]
pub struct IndicatifProgress {
    multi: MultiProgress,
    bars: std::sync::Mutex<std::collections::HashMap<String, ProgressBar>>,
}

#[cfg(feature = "progress")]
impl IndicatifProgress {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            bars: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(feature = "progress")]
impl Default for IndicatifProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "progress")]
impl ProgressTracker for IndicatifProgress {
    fn start_download(&self, digest: &str, total_bytes: u64) {
        let pb = self.multi.add(ProgressBar::new(total_bytes));
        if let Ok(style) = ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        {
            pb.set_style(style.progress_chars("#>-"));
        }
        pb.set_message(format!("{}..", &digest[..8.min(digest.len())]));
        self.bars.lock().unwrap().insert(digest.to_string(), pb);
    }

    fn update(&self, digest: &str, bytes: u64) {
        if let Some(pb) = self.bars.lock().unwrap().get(digest) {
            pb.inc(bytes);
        }
    }

    fn finish(&self, digest: &str) {
        if let Some(pb) = self.bars.lock().unwrap().remove(digest) {
            pb.finish_with_message("Done");
        }
    }
}
