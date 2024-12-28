use anyhow::Context;
use chrono::{Datelike, Timelike};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use tracing::debug;

pub struct ShortTime;

impl tracing_subscriber::fmt::time::FormatTime for ShortTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now();

        write!(
            w,
            "{:02}:{:02}:{:02} => ",
            now.hour(),
            now.minute(),
            now.second()
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

pub fn format_repo_data(repo_user: &str, repo_name: &str) -> String {
    format!(
        "User => {}\nRepo => {}\n\nURL => https://github.com/{}/{}.git",
        repo_user, repo_user, repo_user, repo_name
    )
    .to_string()
}
