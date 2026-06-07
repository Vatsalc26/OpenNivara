use super::types::{ModelRequest, ModelResponse};
use std::future::Future;
use std::pin::Pin;

pub type ModelProviderFuture<'a> =
    Pin<Box<dyn Future<Output = anyhow::Result<ModelResponse>> + Send + 'a>>;

pub trait ModelProvider {
    fn generate<'a>(&'a self, request: ModelRequest) -> ModelProviderFuture<'a>;
}
