mod api_models;
mod cached_client;
mod client;
pub mod oauth2;
mod token_store;

pub mod cache;

pub use cached_client::{CachedSpotifyClient, SpotifyApiClient, SpotifyResult};
pub use client::SpotifyApiError;
pub use token_store::TokenStore;

pub async fn clear_user_cache() -> Option<()> {
    cache::CacheManager::for_dir("spotty/net")?
        .clear_cache_pattern(&cached_client::USER_CACHE)
        .await
        .ok()
}
