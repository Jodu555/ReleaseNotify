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
