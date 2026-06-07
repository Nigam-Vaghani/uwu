use async_trait::async_trait;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, String>;
}
