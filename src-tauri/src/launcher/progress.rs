use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
pub struct ProgressPayload {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub files_done: u64,
    pub files_total: u64,
}

pub struct Progress {
    downloaded_bytes: u64,
    files_done: u64,
    total_bytes: u64,
    files_total: u64,
    event: &'static str,
}

impl Progress {
    pub fn new(total_bytes: u64, files_total: u64) -> Self {
        Self::with_event(total_bytes, files_total, "install-progress")
    }

    pub fn with_event(total_bytes: u64, files_total: u64, event: &'static str) -> Self {
        Self {
            downloaded_bytes: 0,
            files_done: 0,
            total_bytes,
            files_total,
            event,
        }
    }

    pub fn add_file(&mut self, app: &AppHandle, size: u64) {
        self.downloaded_bytes += size;
        self.files_done += 1;
        self.emit(app);
    }

    pub fn add_bytes(&mut self, app: &AppHandle, bytes: u64) {
        self.downloaded_bytes += bytes;
        self.emit(app);
    }

    pub fn finish_file(&mut self, app: &AppHandle) {
        self.files_done += 1;
        self.emit(app);
    }

    fn emit(&self, app: &AppHandle) {
        let _ = app.emit(
            self.event,
            ProgressPayload {
                downloaded_bytes: self.downloaded_bytes,
                total_bytes: self.total_bytes,
                files_done: self.files_done,
                files_total: self.files_total,
            },
        );
    }
}
