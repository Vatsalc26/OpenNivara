use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ModelContextInfo {
    pub provider: String,
    pub model_name: String,
    pub context_window_tokens: u32,
    pub default_reserved_output_tokens: u32,
    pub supports_token_counting: bool,
    pub supports_usage_metadata: bool,
    pub tokenizer_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct ModelUsageMetadata {
    pub provider: String,
    pub model_name: String,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub recorded_at: String,
}

pub fn get_current_model_context_info() -> ModelContextInfo {
    get_model_context_info("gemini", "gemini-2.5-flash")
}

pub fn get_model_context_info(provider: &str, model: &str) -> ModelContextInfo {
    match (provider, model) {
        ("gemini", "gemini-2.5-flash") => ModelContextInfo {
            provider: "gemini".into(),
            model_name: "gemini-2.5-flash".into(),
            context_window_tokens: 1_000_000,
            default_reserved_output_tokens: 8_192,
            supports_token_counting: false,
            supports_usage_metadata: true,
            tokenizer_strategy: "local_estimate".into(),
        },
        ("gemini", model_name) => ModelContextInfo {
            provider: "gemini".into(),
            model_name: model_name.into(),
            context_window_tokens: 128_000,
            default_reserved_output_tokens: 4_096,
            supports_token_counting: false,
            supports_usage_metadata: true,
            tokenizer_strategy: "local_estimate".into(),
        },
        (provider_name, model_name) => ModelContextInfo {
            provider: provider_name.into(),
            model_name: model_name.into(),
            context_window_tokens: 32_000,
            default_reserved_output_tokens: 2_048,
            supports_token_counting: false,
            supports_usage_metadata: false,
            tokenizer_strategy: "fallback_estimate".into(),
        },
    }
}

pub fn estimate_tokens(text: &str, _model: &ModelContextInfo) -> u32 {
    ((text.chars().count() as f64) / 4.0).ceil().max(1.0) as u32
}

pub fn maybe_count_tokens_with_provider(
    text: &str,
    model: &ModelContextInfo,
) -> anyhow::Result<Option<u32>> {
    if model.supports_token_counting {
        Ok(Some(estimate_tokens(text, model)))
    } else {
        Ok(None)
    }
}

pub fn record_usage_metadata(
    provider: &str,
    model_name: &str,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
) -> ModelUsageMetadata {
    ModelUsageMetadata {
        provider: provider.into(),
        model_name: model_name.into(),
        input_tokens,
        output_tokens,
        total_tokens: input_tokens
            .zip(output_tokens)
            .map(|(input, output)| input + output),
        recorded_at: chrono::Utc::now().to_rfc3339(),
    }
}
