use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;

use crate::{Config, misc};

pub async fn handle_release_github(
    config: Arc<Config>,
    state: Arc<RwLock<serde_json::Value>>,
    release_name: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let previous_id = misc::prepare_handle_release(&state, &release_name).await;

    let feedurl = format!("https://github.com/{}/releases.atom", release_name);
    let content = reqwest::get(feedurl).await?.bytes().await?;
    let parsed_feed = better_feed::parser::parse(&content[..])?;

    let latest_item = &parsed_feed.entries[0];

    let current_id = &latest_item.id;

    let link = &latest_item.links.get(0);

    let title = &latest_item.title;
    let title_text: String;
    if let Some(title) = title {
        title_text = format!("{}\n", title.content)
    } else {
        title_text = "".to_string();
    }

    let message: String;
    if let Some(link) = link {
        message = format!("{}{}", title_text, link.href)
    } else {
        message = format!("{}https://github.com/{}", title_text, &release_name)
    }

    misc::debrief_handle_release(
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
