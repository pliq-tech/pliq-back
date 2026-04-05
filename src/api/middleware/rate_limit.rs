use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;

use crate::api::errors::ApiError;

/// Tracks token bucket state: (last_refill_time, available_tokens).
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<IpAddr, (Instant, f64)>>,
    rps: f64,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            rps: requests_per_second as f64,
        }
    }

    fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut entry = self.buckets.entry(ip).or_insert((now, self.rps));
        let (last_refill, tokens) = entry.value_mut();

        let elapsed = now.duration_since(*last_refill).as_secs_f64();
        let refilled = (*tokens + elapsed * self.rps).min(self.rps);
        *last_refill = now;

        if refilled >= 1.0 {
            *tokens = refilled - 1.0;
            true
        } else {
            *tokens = refilled;
            false
        }
    }

    /// Remove entries older than 60 seconds to prevent unbounded growth.
    pub fn cleanup(&self) {
        let cutoff = Instant::now() - std::time::Duration::from_secs(60);
        self.buckets.retain(|_, (last, _)| *last > cutoff);
    }
}

pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let limiter = req
        .extensions()
        .get::<RateLimiter>()
        .cloned();

    if let Some(limiter) = limiter {
        if !limiter.check(addr.ip()) {
            return ApiError::RateLimited.into_response();
        }
    }

    next.run(req).await
}

/// Spawn a background task that periodically cleans up stale entries.
pub fn spawn_cleanup_task(limiter: RateLimiter) {
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            limiter.cleanup();
        }
    });
}
