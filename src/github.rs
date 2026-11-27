use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::{Config, send_notification};

pub async fn prepare_handle_release(
    state: &Arc<RwLock<serde_json::Value>>,
    release_name: &String,
) -> String {
    let previous_id = {
        let state = state.read().await;
        state
            .get(&release_name)
            .and_then(|v| v.as_str())
            .map(String::from)
    };
    match previous_id {
        Some(previous_id) => previous_id,
        _ => {
            println!("No previous release found for {}.", release_name);
            //If we have no previous id we default to an empty String
            "".to_string()
        }
    }
}

pub async fn debrief_handle_release(
    state: &Arc<RwLock<serde_json::Value>>,
    config: &Arc<Config>,
    release_name: &String,
    previous_id: &String,
    current_id: &String,
    message: &String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    //                               \/ If we did not have a previous id we ignore
    let is_new_release = previous_id != "" && *previous_id != *current_id;
    if !is_new_release {
        println!(
            "No new release for {}. prev: {} curr: {}",
            release_name, previous_id, current_id,
        );
        return Ok(());
    } else {
        println!(
            "New release found for {}! prev: {} curr: {}",
            &release_name, previous_id, current_id,
        );
        let title = format!("New release: {}", &release_name);
        send_notification(&config, &title, &message)
            .await
            .expect("Failed to send Gotify Message")
    }

    {
        let mut state = state.write().await;
        state[&release_name] = serde_json::Value::String(current_id.clone());
    }
    Ok(())
}

pub async fn handle_release_github(
    config: Arc<Config>,
    state: Arc<RwLock<serde_json::Value>>,
    release_name: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let previous_id = prepare_handle_release(&state, &release_name).await;

    let feedurl = format!("https://github.com/{}/releases.atom", release_name);
    let content = reqwest::get(feedurl).await?.bytes().await?;
    let parsed_feed = better_feed::parser::parse(&content[..])?;

    let latest_item = &parsed_feed.entries[0];

    let current_id = &latest_item.id;

    let link = &latest_item.links.get(0);

    let message: String;
    if let Some(link) = link {
        message = link.href.to_owned();
    } else {
        message = format!("https://github.com/{}", &release_name)
    }

    debrief_handle_release(
        &state,
        &config,
        &release_name,
        &previous_id,
        &current_id,
        &message,
    )
    .await?;
    Ok(())
}
