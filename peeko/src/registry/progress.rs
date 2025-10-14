//! Progress reporting abstraction used when downloading blobs from the registry.

#[cfg(feature = "progress")]
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

/// Trait implemented by download progress reporters.
pub trait ProgressTracker: Send + Sync {
    fn start_download(&self, digest: &str, total_bytes: u64);
    fn update(&self, digest: &str, bytes: u64);
    fn finish(&self, digest: &str);
}

/// No-op progress tracker used when no reporting is required.
pub struct NoopProgress;

impl ProgressTracker for NoopProgress {
    fn start_download(&self, _digest: &str, _total_bytes: u64) {}
    fn update(&self, _digest: &str, _bytes: u64) {}
    fn finish(&self, _digest: &str) {}
}

/// Progress tracker backed by `indicatif` progress bars (only available when the
/// `progress` feature is enabled).
#[cfg(feature = "progress")]
pub struct IndicatifProgress {
    multi: MultiProgress,
    bars: std::sync::Mutex<std::collections::HashMap<String, ProgressBar>>,
}

#[cfg(feature = "progress")]
impl IndicatifProgress {
    /// Creates a new progress reporter wired to a `MultiProgress` manager.
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
        let digest_display = if let Some(pos) = digest.find(":") {
            &digest[pos + 1..(pos + 12).min(digest.len())]
        } else {
            &digest[..12.min(digest.len())]
        };
        pb.set_message(format!("{digest_display}.."));
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
