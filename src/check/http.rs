use super::{Check, CheckFuture};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

pub struct HttpCheck {
    url: String,
    description: String,
    client: Arc<Client>,
}

impl HttpCheck {
    pub fn new(url: String, client: Arc<Client>) -> Self {
        let description = format!("http:{url}");
        Self {
            url,
            description,
            client,
        }
    }
}

pub fn build_http_client(timeout: Duration) -> Result<Client, reqwest::Error> {
    Client::builder().timeout(timeout).build()
}

impl Check for HttpCheck {
    fn check(&self) -> CheckFuture<'_> {
        Box::pin(async move {
            match self.client.get(&self.url).send().await {
                Ok(resp) if resp.status().is_success() => Ok(()),
                Ok(resp) => Err(format!(
                    "http {} returned status {}",
                    self.url,
                    resp.status()
                )),
                Err(e) => Err(format!("http {}: {e}", self.url)),
            }
        })
    }

    fn description(&self) -> &str {
        &self.description
    }
}
