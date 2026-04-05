//! gRPC client stubs for communicating with pliq-ai service.
//!
//! Uses tonic-generated client code from proto compilation.
//! All methods are stubs that log warnings until the AI service
//! is connected.

/// Proto-generated types for the AI service.
pub mod ai_service {
    tonic::include_proto!("pliq.ai.v1");
}

// Note: pliq.common.v1 types will be available once build.rs
// compiles common.proto alongside ai_service.proto.

/// Client wrapper for the pliq-ai gRPC service.
pub struct AiClient {
    ai_host: String,
}

impl AiClient {
    pub fn new(ai_host: String) -> Self {
        Self { ai_host }
    }

    /// Analyze a listing image for fraud indicators.
    pub async fn analyze_image(
        &self,
        _request: ai_service::ImageAnalysisRequest,
    ) -> Result<ai_service::ImageAnalysisResponse, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: analyze_image is a stub"
        );
        Ok(ai_service::ImageAnalysisResponse::default())
    }

    /// Analyze an entire listing for fraud risk.
    pub async fn analyze_listing(
        &self,
        _request: ai_service::ListingAnalysisRequest,
    ) -> Result<ai_service::ListingAnalysisResponse, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: analyze_listing is a stub"
        );
        Ok(ai_service::ListingAnalysisResponse::default())
    }

    /// Check price reasonability for a listing.
    pub async fn check_price_reasonability(
        &self,
        _request: ai_service::PriceCheckRequest,
    ) -> Result<ai_service::PriceCheckResponse, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: check_price_reasonability is a stub"
        );
        Ok(ai_service::PriceCheckResponse::default())
    }

    /// Rank listings for a tenant by compatibility.
    pub async fn rank_listings(
        &self,
        _request: ai_service::RankRequest,
    ) -> Result<ai_service::RankResponse, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: rank_listings is a stub"
        );
        Ok(ai_service::RankResponse::default())
    }

    /// Recommend listings for a tenant.
    pub async fn recommend_listings(
        &self,
        _request: ai_service::RecommendRequest,
    ) -> Result<ai_service::RecommendResponse, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: recommend_listings is a stub"
        );
        Ok(ai_service::RecommendResponse::default())
    }
}

/// Errors from the AI gRPC client.
#[derive(Debug, thiserror::Error)]
pub enum AiClientError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),
}
