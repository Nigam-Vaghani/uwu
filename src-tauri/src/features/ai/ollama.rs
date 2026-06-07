use super::provider::AiProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
        }
    }

    fn endpoint(&self) -> String {
        format!("{}/api/generate", self.base_url)
    }
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn complete(&self, prompt: &str) -> Result<String, String> {
        let request = GenerateRequest {
            model: &self.model,
            prompt,
            stream: false,
            options: GenerateOptions {
                temperature: 0.8,
                num_predict: 80,
            },
        };

        let response = self
            .client
            .post(self.endpoint())
            .json(&request)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Ollama API error ({status}): {body}"));
        }

        let payload = response
            .json::<GenerateResponse>()
            .await
            .map_err(|error| error.to_string())?;

        let text = payload.response.trim().to_string();
        if text.is_empty() {
            return Err("Ollama returned an empty response".to_string());
        }

        Ok(text)
    }
}
