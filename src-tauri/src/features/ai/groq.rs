use super::provider::AiProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const GROQ_URL: &str = "https://api.groq.com/openai/v1/chat/completions";

pub struct GroqProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GroqProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

#[async_trait]
impl AiProvider for GroqProvider {
    async fn complete(&self, prompt: &str) -> Result<String, String> {
        let request = ChatRequest {
            model: &self.model,
            messages: vec![ChatMessage {
                role: "user",
                content: prompt,
            }],
            max_tokens: 80,
            temperature: 0.8,
        };

        let response = self
            .client
            .post(GROQ_URL)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Groq API error ({status}): {body}"));
        }

        let payload = response
            .json::<ChatResponse>()
            .await
            .map_err(|error| error.to_string())?;

        payload
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content.trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| "Groq returned an empty response".to_string())
    }
}
