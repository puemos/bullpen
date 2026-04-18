pub mod provider;
pub mod providers;
pub mod registry;

pub use provider::{
    ProviderCallContext, ProviderDescriptor, SourceCategory, SourceError, SourceProvider,
};
pub use registry::{all, get, http_client};

/// Keychain account identifier for a provider's single API-key slot.
#[must_use]
pub fn key_account(provider_id: &str) -> String {
    format!("source.{provider_id}.api_key")
}

/// Environment variable the MCP child reads to pick up a provider's API key.
#[must_use]
pub fn key_env_var(provider_id: &str) -> String {
    format!("BULLPEN_SRC_KEY_{}", provider_id.to_uppercase())
}
