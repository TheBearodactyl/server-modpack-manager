mod utils;

use {
    anyhow::{Context, Result},
    crossterm::event::{self, Event, KeyCode},
    octocrab::Octocrab,
    reqwest::{self, Url},
    std::{
        env,
        fs::{self, read_to_string},
        io::{self, Write},
        path::PathBuf,
        str::FromStr,
    },
    tokio::main,
    tracing::{debug, error, info},
    zip::ZipArchive,
};

#[main]
async fn main() -> Result<()> {
    utils::logging();

    debug!("Starting application...");

    let octocrab = Octocrab::builder()
        .build()
        .context("Failed to build Octocrab client")?;
    let repo = octocrab.repos("thebearodactyl", "originalife-s4");
    let latest_release = repo
        .releases()
        .get_latest()
        .await
        .context("Failed to fetch latest release from GitHub")?;

    info!("Latest release fetched: {}", latest_release.tag_name);

    println!("Which launcher do you use?");
    println!("1. Modrinth");
    println!("2. CurseForge");
    println!("3. Prism");
    io::stdout().flush().context("Failed to flush stdout")?;

    let choice = loop {
        if let Ok(Event::Key(key_event)) = event::read() {
            match key_event.code {
                KeyCode::Char('1') => break "1",
                KeyCode::Char('2') => break "2",
                KeyCode::Char('3') => break "3",
                _ => println!("Please press 1, 2, or 3."),
            }
        }
    };

    let artifact_name = match choice {
        "1" => "updated-pack-modrinth.zip",
        "2" => "updated-pack-curseforge.zip",
        _ => "updated-pack-prism.zip",
    };

    if let Some(asset) = latest_release
        .assets
        .iter()
        .find(|a| a.name == artifact_name)
    {
        info!("Found asset: {}", artifact_name);

        let client = reqwest::Client::new();
        let url = Url::from_str(asset.browser_download_url.as_str()).context("Invalid URL")?;
        let total_size = asset.size;

        let pb = utils::progress_bar(total_size);

        let mut response = client
            .get(url)
            .send()
            .await
            .context("Failed to send request to download asset")?;

        let mut content = Vec::with_capacity(total_size as usize);

        while let Some(chunk) = response
            .chunk()
            .await
            .context("Failed to read chunk during download")?
        {
            content.extend_from_slice(&chunk);
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Download completed");

        let appdata_path = env::var("APPDATA").context("Failed to get APPDATA")?;
        let zip_target_dir = PathBuf::from(&appdata_path)
            .join("Originalife Season 4")
            .join(latest_release.tag_name.clone());

        if !zip_target_dir.exists() {
            fs::create_dir_all(&zip_target_dir)
                .context("Failed to create AppData target directory")?;
        }

        let temp_file = zip_target_dir.join(artifact_name);
        fs::write(&temp_file, content).context("Failed to write ZIP file to AppData")?;

        let profile_dir = match choice {
            "1" => env::var("APPDATA").context("Failed to get APPDATA")? + r"\ModrinthApp\profiles",
            "2" => {
                env::var("HOMEDRIVE").context("Failed to get HOMEDRIVE")?
                    + &env::var("HOMEPATH").context("Failed to get HOMEPATH")?
                    + r"\curseforge\minecraft\Instances"
            }
            "3" => {
                env::var("APPDATA").context("Failed to get APPDATA")? + r"\PrismLauncher\instances"
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid choice"))
                    .context("User made an invalid choice")?
            }
        };

        let target_dir = PathBuf::from(&profile_dir).join(format!(
            "Originalife Season 4 - {}",
            latest_release.tag_name
        ));

        if target_dir.exists() {
            utils::remove_dir_contents(&target_dir)
                .context("Failed to clean mod manager target directory")?;
        } else {
            fs::create_dir_all(&target_dir)
                .context("Failed to create mod manager target directory")?;
        }

        let file = fs::File::open(&temp_file).context("Failed to open ZIP file from AppData")?;
        let mut archive = ZipArchive::new(file).context("Failed to create ZIP archive")?;
        archive
            .extract(&target_dir)
            .context("Failed to extract ZIP archive")?;

        // Spawn a separate task to update the JSON after 10 seconds
        let json_path_clone = target_dir.join("minecraftinstance.json");
        
        if json_path_clone.exists() {
            info!("Removing existing minecraftinstance.json");
            fs::remove_file(&json_path_clone).context("Failed to remove minecraftinstance.json")?;
        }
        
        let json_path_str = json_path_clone.to_str().unwrap().to_string();
        let latest_release_tag = latest_release.tag_name.clone();
        info!("Updating {json_path_str}");

        if json_path_clone.exists() {
            match utils::update_json_value(
                json_path_clone
                    .as_path()
                    .to_str()
                    .expect("Couldn't find instance json file"),
                &["manifest", "name"],
                format!("Originalife Season 4 - {}", latest_release_tag).as_str(),
                Some(5u64)
            ).await {
                Ok(_) => info!("(1/2) Update of minecraftinstance.json completed"),
                Err(e) => error!("Failed to update minecraftinstance.json: {}", e),
            }

            match utils::update_json_value(
                json_path_clone
                    .as_path()
                    .to_str()
                    .expect("Couldn't find instance json file"),
                &["name"],
                format!("Originalife Season 4 - {}", latest_release_tag).as_str(),
                Some(5u64)
            ).await {
                Ok(_) => info!("(2/2) Update of minecraftinstance.json completed"),
                Err(e) => error!("Failed to update minecraftinstance.json: {}", e),
            }
        }

        if choice == "3" {
            let instance_cfg_path = target_dir.join("instance.cfg");
            if instance_cfg_path.exists() {
                let mut instance_cfg =
                    read_to_string(&instance_cfg_path).context("Failed to read instance.cfg")?;
                instance_cfg = regex::Regex::new(r"(?m)^name=.*$")?
                    .replace_all(&instance_cfg, |_: &regex::Captures| {
                        format!("name=Originalife Season 4 - {}", latest_release.tag_name)
                    })
                    .to_string();
                fs::write(instance_cfg_path, instance_cfg)
                    .context("Failed to update instance.cfg")?;
            }
        }

        info!("Update completed successfully!");
    } else {
        error!("No new release found or '{}' not available.", artifact_name);
        println!("No new release found or '{}' not available.", artifact_name);
    }

    Ok(())
}
