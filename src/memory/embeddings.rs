use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EmbeddingStatus {
    pub vector_feature_enabled: bool,
    pub local_embeddings_enabled: bool,
    pub provider: String,
}

pub fn status() -> EmbeddingStatus {
    EmbeddingStatus {
        vector_feature_enabled: cfg!(feature = "memory-vector"),
        local_embeddings_enabled: cfg!(feature = "local-embeddings"),
        provider: if cfg!(feature = "local-embeddings") {
            "fastembed"
        } else if cfg!(feature = "memory-vector") {
            "external_embeddings_required"
        } else {
            "disabled"
        }
        .to_string(),
    }
}
