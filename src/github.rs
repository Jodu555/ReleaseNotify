use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::{Config, send_notification};

pub async fn handle_release_github(
    config: Arc<Config>,
    state: Arc<RwLock<serde_json::Value>>,
    release_name: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let previous_id = {
        let state = state.read().await;
        state
            .get(&release_name)
            .and_then(|v| v.as_str())
            .map(String::from)
    };
    if previous_id.is_none() {
        println!("No previous release found for {}.", release_name);
    }
    let feedurl = format!("https://github.com/{}/releases.atom", release_name);
    let content = reqwest::get(feedurl).await?.bytes().await?;
    let parsed_feed = better_feed::parser::parse(&content[..])?;

    let latest_item = &parsed_feed.entries[0];

    let current_id = &latest_item.id;

    if let Some(prev_id) = previous_id {
        if prev_id == *current_id {
            println!(
                "No new release for {}. prev: {} curr: {}",
                release_name, prev_id, current_id,
            );
            return Ok(());
        } else {
            println!(
                "New release found for {}! prev: {} curr: {}",
                &release_name, prev_id, current_id,
            );
            let title = format!("New release: {}", &release_name);
            let message = format!("{}/{}", &latest_item.id, &latest_item.id);
            send_notification(&config, &title, &message)
                .await
                .expect("Failed to send Gotify Message")
        }
    }
    {
        let mut state = state.write().await;
        state[&release_name] = serde_json::Value::String(latest_item.id.clone());
    }
    Ok(())
}
