use fs::write;
use anyhow::Context;
use chrono::{Datelike, Timelike};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::{from_str, to_string_pretty, Value};
use std::fs;
use std::fs::read_to_string;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error};

pub struct ShortTime;

impl tracing_subscriber::fmt::time::FormatTime for ShortTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now();

        write!(
            w,
            "{:02}:{:02}:{:02} - {:02}/{:02}/{:02}",
            now.hour(),
            now.minute(),
            now.second(),
            now.day(),
            now.month(),
            now.year()
        )
    }
}

pub fn remove_dir_contents<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    debug!("Removing contents of directory: {:?}", path.as_ref());
    for entry in fs::read_dir(path).context("Failed to read directory")? {
        let entry = entry.context("Failed to read entry")?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).context(format!("Failed to remove directory: {:?}", path))?;
        } else {
            fs::remove_file(&path).context(format!("Failed to remove file: {:?}", path))?;
        }
    }
    Ok(())
}

pub fn progress_bar(total_size: i64) -> ProgressBar {
    let pb = ProgressBar::new(total_size as u64);

    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.pink} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .expect("Invalid progress bar template")
        .progress_chars("=> ")
    );
    
    pb
}

pub async fn update_json_value(
    file_path: &str,
    key_path: &[&str],
    new_value: &str,
    delay_seconds: Option<u64>,
) -> anyhow::Result<()> {
    if let Some(delay) = delay_seconds {
        sleep(Duration::from_secs(delay)).await;
    }

    let json_content: String = read_to_string(file_path).context("Failed to read JSON file")?;
    let mut json: Value = from_str(&json_content).context("Failed to parse JSON")?;

    debug!("Initial JSON structure: {:?}", json);

    let current = &mut json;
    if let Some(value) = edit_json(key_path, new_value, current) {
        return value;
    }

    debug!("Updated JSON structure: {:?}", json);
    let updated_json = to_string_pretty(&json).context("Failed to serialize JSON")?;
    write(file_path, updated_json).context("Failed to write updated JSON")?;

    Ok(())
}

fn edit_json(
    key_path: &[&str],
    new_value: &str,
    mut current: &mut Value,
) -> Option<anyhow::Result<()>> {
    for (i, &key) in key_path.iter().enumerate() {
        debug!("Traversing key: {}", key);

        if i == key_path.len() - 1 {
            if let Some(obj) = current.as_object_mut() {
                if obj.contains_key(key) {
                    debug!("Updating key '{}' with new value: {}", key, new_value);
                    obj.insert(key.to_string(), Value::String(new_value.to_string()));
                } else {
                    error!("Key '{}' not found in the JSON structure", key);
                    return Some(Err(anyhow::anyhow!("Key not found in the JSON structure")));
                }
            }
        } else {
            match current.get_mut(key) {
                Some(next) => {
                    debug!("Found key '{}', moving deeper", key);
                    current = next;
                }
                None => {
                    error!("Failed to find nested key '{}'", key);
                    return Some(Err(anyhow::anyhow!("Failed to find nested key")));
                }
            }
        }
    }
    None
}

pub fn logging() {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_timer(ShortTime)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::fmt()
            .with_timer(ShortTime)
            .with_max_level(tracing::Level::INFO)
            .init();
    }
}
