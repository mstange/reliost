use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use samply_quota_manager::QuotaManagerNotifier;
use wholesym::{DownloadError, SymbolManagerObserver};

pub struct QuotaManagingSymbolManagerObserver {
    quota_manager_notifiers: Vec<QuotaManagerNotifier>,
    urls: Mutex<HashMap<u64, String>>,
}

impl QuotaManagingSymbolManagerObserver {
    pub fn new(quota_manager_notifiers: Vec<QuotaManagerNotifier>) -> Self {
        Self {
            quota_manager_notifiers,
            urls: Mutex::new(HashMap::new()),
        }
    }
}

impl SymbolManagerObserver for QuotaManagingSymbolManagerObserver {
    fn on_new_download_before_connect(&self, download_id: u64, url: &str) {
        tracing::info!(url, "Connecting to URL");
        self.urls
            .lock()
            .unwrap()
            .insert(download_id, url.to_owned());
    }

    fn on_download_started(&self, download_id: u64) {
        let url = self.urls.lock().unwrap().get(&download_id).unwrap().clone();
        tracing::info!(url, "Downloading from URL");
    }

    fn on_download_progress(
        &self,
        _download_id: u64,
        _bytes_so_far: u64,
        _total_bytes: Option<u64>,
    ) {
    }

    fn on_download_completed(
        &self,
        download_id: u64,
        uncompressed_size_in_bytes: u64,
        time_until_headers: std::time::Duration,
        time_until_completed: std::time::Duration,
    ) {
        let url = self.urls.lock().unwrap().remove(&download_id).unwrap();
        tracing::info!(
            url,
            uncompressed_size_in_bytes,
            time_until_headers_in_seconds = time_until_headers.as_secs_f64(),
            time_until_completed_in_seconds = time_until_completed.as_secs_f64(),
            "Finished download from URL"
        );
    }

    fn on_download_failed(&self, download_id: u64, reason: DownloadError) {
        let url = self.urls.lock().unwrap().remove(&download_id).unwrap();
        tracing::info!(
            url,
            reason = reason.to_string(),
            "Failed to download from URL"
        );
    }

    fn on_download_canceled(&self, download_id: u64) {
        let url = self.urls.lock().unwrap().remove(&download_id).unwrap();
        tracing::info!(url, "Canceled download from URL");
    }

    fn on_file_created(&self, path: &Path, size_in_bytes: u64) {
        tracing::info!(
            path = path.to_string_lossy().to_string(),
            size_in_bytes,
            "Created new file"
        );
        for notifier in &self.quota_manager_notifiers {
            notifier.on_file_created(path, size_in_bytes, SystemTime::now());
            notifier.trigger_eviction_if_needed();
        }
    }

    fn on_file_accessed(&self, path: &Path) {
        tracing::info!(path = path.to_string_lossy().to_string(), "File accessed");
        for notifier in &self.quota_manager_notifiers {
            notifier.on_file_accessed(path, SystemTime::now());
        }
    }

    fn on_file_missed(&self, path: &Path) {
        tracing::info!(
            path = path.to_string_lossy().to_string(),
            "File access missed"
        );
    }
}
