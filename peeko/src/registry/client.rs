use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct RegistryClient {
    client: Client,
    registry_url: String,
    auth_token: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl RegistryClient {
    pub fn new(registry_url: &str) -> Self {
        Self {
            client: Client::new(),
            registry_url: registry_url.to_string(),
            auth_token: None,
            username: None,
            password: None,
        }
    }

    pub fn with_credentials(registry_url: &str, username: &str, password: &str) -> Self {
        let mut client = Self::new(registry_url);
        client.username = Some(username.to_string());
        client.password = Some(password.to_string());
        client
    }

    pub fn with_token(registry_url: &str, token: &str) -> Self {
        let mut client = Self::new(registry_url);
        client.auth_token = Some(token.to_string());
        client
    }
}
