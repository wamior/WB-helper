use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
pub struct ServerVersionInfo {
    pub version: String,
    pub url: String,
}

pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
}

pub fn check_for_updates(update_url: &str) -> anyhow::Result<UpdateInfo> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    
    let client = reqwest::blocking::Client::builder()
        .user_agent("pvzhelper-updater")
        .build()?;
        
    let info: ServerVersionInfo = client.get(update_url).send()?.json()?;
    
    Ok(UpdateInfo {
        current_version,
        latest_version: info.version,
        download_url: Some(info.url),
    })
}

pub fn download_and_replace(url: &str) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("pvzhelper-updater")
        .build()?;
        
    let response = client.get(url).send()?;
    let bytes = response.bytes()?;
    
    let temp_exe = env::temp_dir().join("pvzhelper_update.exe");
    std::fs::write(&temp_exe, bytes)?;
    
    self_replace::self_replace(&temp_exe)?;
    std::fs::remove_file(&temp_exe).ok();
    
    Ok(())
}
pub fn restart_app() {
    if let Ok(current_exe) = env::current_exe() {
        std::process::Command::new(current_exe)
            .spawn()
            .ok();
        std::process::exit(0);
    }
}

pub fn cleanup_old_executables() {
    let mut paths_to_check = Vec::new();
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            paths_to_check.push(parent.to_path_buf());
        }
    }
    if let Ok(cwd) = env::current_dir() {
        paths_to_check.push(cwd);
    }
    
    // Deduplicate
    paths_to_check.sort();
    paths_to_check.dedup();

    for dir in paths_to_check {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let is_temp = file_name.contains("__selfdelete__") 
                        || file_name.ends_with(".tmp") 
                        || (file_name.starts_with(".") && file_name.ends_with(".exe"));
                    
                    if is_temp {
                        // Avoid deleting the current running exe if it matches the pattern
                        if let Ok(cur_exe) = env::current_exe() {
                            if path == cur_exe { continue; }
                        }
                        
                        // Retrying a few times because the old process might still be exiting
                        // and holding a lock for a moment.
                        for _ in 0..5 {
                            if std::fs::remove_file(&path).is_ok() {
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(200));
                        }
                    }
                }
            }
        }
    }
}
