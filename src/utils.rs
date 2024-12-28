use anyhow::{Context, Error};
use chrono::Timelike;
use indicatif::{ProgressBar, ProgressStyle};
use octocrab::models::repos::Release;
use octocrab::Octocrab;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

pub struct ShortTime;

impl tracing_subscriber::fmt::time::FormatTime for ShortTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now();

        write!(
            w,
            "=>"
            //now.hour(),
            //now.minute(),
            //now.second()
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

pub async fn latest_release(repo_user: &str, repo_name: &str) -> anyhow::Result<Release, Error> {
    let octocrab = Octocrab::builder()
        .build()
        .context("Failed to build Octocrab client")?;
    let repo = octocrab.repos(repo_user, repo_name);

    let selected_release = repo
        .releases()
        .get_latest()
        .await
        .context("Failed to fetch latest release from GitHub")?;

    info!("Latest release fetched: {}", selected_release.tag_name);
    Ok(selected_release)
}
