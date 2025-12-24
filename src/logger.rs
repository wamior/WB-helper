use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

#[derive(Clone)]
pub struct Logger {
    log_dir: PathBuf,
}

impl Logger {
    pub fn new() -> Self {
        let log_dir = if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata).join("PVZHelper").join("logs")
        } else {
            PathBuf::from("logs")
        };

        let _ = create_dir_all(&log_dir);

        Self { log_dir }
    }

    pub fn append(&self, message: &str) {
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H:%M:%S").to_string();
        let file_path = self.log_dir.join(format!("{}.log", date_str));

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
        {
            let _ = writeln!(file, "[{}] {}", time_str, message);
        }
    }
}
