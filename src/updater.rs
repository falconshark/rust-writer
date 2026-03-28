// updater.rs — Background GitHub release checker

use std::sync::mpsc;

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/falconshark/rust-writer/releases/latest";
const GITHUB_RELEASES_URL: &str =
    "https://github.com/falconshark/rust-writer/releases";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum UpdateStatus {
    UpdateAvailable(String), // new version string
    UpToDate,
    Failed,
}

pub struct UpdateChecker {
    receiver: mpsc::Receiver<UpdateStatus>,
}

impl UpdateChecker {
    /// Spawn a background thread to check GitHub for the latest release.
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            // Small delay so the app finishes loading before the network hit
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = tx.send(check_latest_version());
        });

        Self { receiver: rx }
    }

    /// Returns Some if the check has completed, None if still in progress.
    pub fn poll(&self) -> Option<UpdateStatus> {
        self.receiver.try_recv().ok()
    }

    /// URL users should visit to download the new version.
    pub fn releases_url() -> &'static str {
        GITHUB_RELEASES_URL
    }
}

fn check_latest_version() -> UpdateStatus {
    let client = match reqwest::blocking::Client::builder()
        .user_agent(concat!("rust-writer/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return UpdateStatus::Failed,
    };

    let response = match client.get(GITHUB_API_URL).send() {
        Ok(r) => r,
        Err(_) => return UpdateStatus::Failed,
    };

    let json: serde_json::Value = match response.json() {
        Ok(j) => j,
        Err(_) => return UpdateStatus::Failed,
    };

    let tag = match json["tag_name"].as_str() {
        Some(t) => t,
        None => return UpdateStatus::Failed,
    };

    // Tags are typically "v0.2.3" — strip the leading 'v'
    let latest = tag.trim_start_matches('v');

    if is_newer(latest, CURRENT_VERSION) {
        UpdateStatus::UpdateAvailable(latest.to_string())
    } else {
        UpdateStatus::UpToDate
    }
}

/// Returns true if `latest` is a higher semver than `current`.
fn is_newer(latest: &str, current: &str) -> bool {
    fn parse(v: &str) -> (u32, u32, u32) {
        let mut parts = v.split('.').filter_map(|p| p.parse::<u32>().ok());
        (
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
            parts.next().unwrap_or(0),
        )
    }
    parse(latest) > parse(current)
}
