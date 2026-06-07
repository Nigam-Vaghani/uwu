use super::provider::AiProvider;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }

    fn endpoint(&self) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        )
    }
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    contents: Vec<Content<'a>>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GenerationConfig {
    max_output_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Option<Vec<CandidatePart>>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: Option<String>,
}

#[async_trait]
impl AiProvider for GeminiProvider {
    async fn complete(&self, prompt: &str) -> Result<String, String> {
        let request = GenerateRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
            generation_config: GenerationConfig {
                max_output_tokens: 80,
                temperature: 0.8,
            },
        };

        let response = self
            .client
            .post(self.endpoint())
            .query(&[("key", self.api_key.as_str())])
            .json(&request)
            .send()
            .await
            .map_err(|error| error.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Gemini API error ({status}): {body}"));
        }

        let payload = response
            .json::<GenerateResponse>()
            .await
            .map_err(|error| error.to_string())?;

        payload
            .candidates
            .unwrap_or_default()
            .into_iter()
            .flat_map(|candidate| candidate.content)
            .flat_map(|content| content.parts.unwrap_or_default())
            .filter_map(|part| part.text)
            .map(|text| text.trim().to_string())
            .find(|text| !text.is_empty())
            .ok_or_else(|| "Gemini returned an empty response".to_string())
    }
}
