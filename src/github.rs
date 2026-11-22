use std::error::Error;

use crate::{Config, send_notification};

pub async fn handle_release_github(
    config: &Config,
    state: &mut serde_json::Value,
    release_name: &str,
) -> Result<(), Box<dyn Error>> {
    let previous_id = state[release_name].as_str();
    if previous_id.is_none() {
        println!("No previous release found for {}.", release_name);
    }
    let feedurl = format!("https://github.com/{}/releases.atom", release_name);
    let content = reqwest::get(feedurl).await?.bytes().await?;
    let parsed_feed = better_feed::parser::parse(&content[..])?;

    let latest_item = &parsed_feed.entries[0];

    let current_id = &latest_item.id;

    if let Some(prev_id) = previous_id {
        if prev_id == current_id {
            println!("No new release for {}.", release_name);
            return Ok(());
        } else {
            println!("New release found for {}!", &release_name);
            let title = format!("New release: {}", &release_name);
            let message = format!("{}/{}", &latest_item.id, &latest_item.id);
            send_notification(config, &title, &message)
                .await
                .expect("Failed to send Gotify Message")
        }
    }
    state[release_name] = serde_json::Value::String(latest_item.id.clone());
    Ok(())
}
