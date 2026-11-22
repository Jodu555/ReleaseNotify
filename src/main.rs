use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{fs, println};

pub mod github;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    gotify_url: String,
    gotify_token: String,
    watch_list: Vec<String>,
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
        let default_config = Config {
            gotify_url: "https://gotify.example.com/".to_string(),
            gotify_token: "YOUT_GOTIFY_TOKEN_HERE".to_string(),
            watch_list: Vec::new(),
        };
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
    let mut state = read_state_file("state.json").expect("Failed to read state file");

    let config = read_config("config.json").expect("Failed to read config");

    dbg!(&config);

    for line in &config.watch_list {
        let (release_type, release_name) = line
            .split_once(" ")
            .expect("String split failed. Remember to follow the pattern <type> <name>");

        match release_type {
            "github" => github::handle_release_github(&config, &mut state, release_name)
                .await
                .expect("Failed to handle GitHub release"),
            _ => println!("Unknown release type: {}", release_type),
        }
    }

    dbg!("Final state: {:?}", &state);

    fs::write("state.json", serde_json::to_string_pretty(&state).unwrap())
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
