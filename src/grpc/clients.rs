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
        _request: ai_service::AnalyzeImageRequest,
    ) -> Result<ai_service::FraudReport, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: analyze_image is a stub"
        );
        Ok(ai_service::FraudReport::default())
    }

    /// Detect price anomalies for a listing.
    pub async fn detect_price_anomaly(
        &self,
        _request: ai_service::PriceAnomalyRequest,
    ) -> Result<ai_service::PriceAnomalyReport, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: detect_price_anomaly is a stub"
        );
        Ok(ai_service::PriceAnomalyReport::default())
    }

    /// Detect duplicate listings.
    pub async fn detect_duplicate(
        &self,
        _request: ai_service::DuplicateListingRequest,
    ) -> Result<ai_service::DuplicateReport, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: detect_duplicate is a stub"
        );
        Ok(ai_service::DuplicateReport::default())
    }

    /// Rank listings for a tenant by compatibility.
    pub async fn rank_listings(
        &self,
        _request: ai_service::RankListingsRequest,
    ) -> Result<ai_service::RankListingsResponse, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: rank_listings is a stub"
        );
        Ok(ai_service::RankListingsResponse::default())
    }

    /// Recommend listings for a tenant.
    pub async fn recommend_listings(
        &self,
        _request: ai_service::RecommendRequest,
    ) -> Result<ai_service::RankListingsResponse, AiClientError> {
        tracing::warn!(
            host = %self.ai_host,
            "AI service not connected: recommend_listings is a stub"
        );
        Ok(ai_service::RankListingsResponse::default())
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
