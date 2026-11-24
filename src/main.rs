use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{dbg, fs, println, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinSet;

pub mod github;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    gotify_url: String,
    gotify_token: String,
    watch_list: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gotify_url: "https://gotify.example.com/".to_string(),
            gotify_token: "YOUT_GOTIFY_TOKEN_HERE".to_string(),
            watch_list: Vec::new(),
        }
    }
}

fn read_state_file(file_path: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    let result = fs::exists(file_path).expect("Failed to check if file exists");
    if !result {
        println!("State file exists: false Creating file...");
        fs::write(file_path, "{}").expect("Unable to write state file");
    } else {
        println!("State file exists: true");
    }

    let string = fs::read_to_string(file_path).expect("Unable to read state file");

    let state =
        serde_json::from_str::<serde_json::Value>(&string).expect("Unable to parse state file");

    Ok(state)
}

fn read_config(file_path: &str) -> Result<Config, Box<dyn Error>> {
    let result = fs::exists(file_path).expect("Failed to check if file exists");
    if !result {
        let default_config = Config::default();
        println!("Creating config file...");
        let config_json_str =
            serde_json::to_string_pretty(&default_config).expect("default config to json failed");
        fs::write(file_path, config_json_str).expect("Unable to write config file");
    }

    let string = fs::read_to_string(file_path).expect("Unable to read state file");

    let config = serde_json::from_str::<Config>(&string).expect("Unable to parse state file");

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = read_state_file("state.json").expect("Failed to read state file");
    let state = Arc::new(RwLock::new(state));

    let config = read_config("config.json").expect("Failed to read config");
    let config = Arc::new(config);

    dbg!(&config);

    let tasks: JoinSet<Result<(), Box<dyn Error + Send + Sync>>> = JoinSet::new();

    for line in &config.watch_list {
        let (release_type, release_name) = line
            .split_once(" ")
            .expect("String split failed. Remember to follow the pattern <type> <name>");

        match release_type {
            "github" => {
                let config_clone = Arc::clone(&config);
                let state_clone = Arc::clone(&state);
                let release_name = release_name.to_string();
                github::handle_release_github(config_clone, state_clone, release_name)
                    .await
                    .expect("Failed to handle GitHub release");
            }
            _ => println!("Unknown release type: {}", release_type),
        }
    }
    for result in tasks.join_all().await {
        result.expect("Task results to succeed ?");
    }

    let final_state = state.read().await;

    dbg!("Final state: {:?}", &final_state);

    fs::write(
        "state.json",
        serde_json::to_string_pretty(&*final_state).unwrap(),
    )
    .expect("Unable to write state file");
    Ok(())
}

async fn send_notification(
    config: &Config,
    title: &str,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Notification - {}: {}", title, message);
    let gotify_url = format!(
        "{}/message?token={}",
        &config.gotify_url, &config.gotify_token
    );
    let client = reqwest::Client::new();
    let _response = client
        .post(&gotify_url)
        .json(&serde_json::json!({
            "title": title,
            "message": message,
            "priority": 5
        }))
        .send()
        .await
        .expect("Failed to send Gotify Message");
    Ok(())
}
