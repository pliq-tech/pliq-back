use axum::middleware as axum_mw;
use axum::routing::{get, post, put};
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::api::handlers;
use crate::api::middleware::auth::auth_middleware;
use crate::api::middleware::cors::cors_layer;
use crate::api::middleware::request_id::request_id_middleware;
use crate::api::middleware::tracing_mw::tracing_middleware;
use crate::AppState;

pub fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/ready", get(handlers::health::ready));

    let auth = Router::new()
        .route("/auth/verify-world-id", post(handlers::auth::verify_world_id));

    let protected = build_protected_routes()
        .layer(axum_mw::from_fn_with_state(state.clone(), auth_middleware));

    let public_reputation = Router::new()
        .route("/reputation/root", get(handlers::reputation::get_merkle_root))
        .route("/reputation/verify-proof", post(handlers::reputation::verify_proof));

    let cors = cors_layer(&state.config.cors_origins);

    Router::new()
        .nest("/api/v1", public)
        .nest("/api/v1", auth)
        .nest("/api/v1", protected)
        .nest("/api/v1", public_reputation)
        .layer(axum_mw::from_fn(tracing_middleware))
        .layer(axum_mw::from_fn(request_id_middleware))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn build_protected_routes() -> Router<AppState> {
    Router::new()
        // Users
        .route("/users/me", get(handlers::users::get_me).put(handlers::users::update_me))
        .route("/users/{id}/public", get(handlers::users::get_public_profile))
        // Listings
        .route("/listings", post(handlers::listings::create_listing).get(handlers::listings::list_listings))
        .route("/listings/{id}", get(handlers::listings::get_listing).put(handlers::listings::update_listing).delete(handlers::listings::delete_listing))
        .route("/listings/{id}/verify", post(handlers::listings::verify_listing))
        .route("/listings/{id}/fraud-score", get(handlers::listings::get_fraud_score))
        // Applications
        .route("/listings/{listing_id}/applications", post(handlers::applications::create_application))
        .route("/applications", get(handlers::applications::list_applications))
        .route("/applications/{id}", get(handlers::applications::get_application).delete(handlers::applications::withdraw_application))
        .route("/applications/{id}/status", put(handlers::applications::update_application_status))
        // Leases
        .route("/leases", post(handlers::leases::create_lease).get(handlers::leases::list_leases))
        .route("/leases/{id}", get(handlers::leases::get_lease))
        .route("/leases/{id}/sign", put(handlers::leases::sign_lease))
        .route("/leases/{id}/terminate", put(handlers::leases::terminate_lease))
        // Payments
        .route("/leases/{lease_id}/payments", post(handlers::payments::create_payment).get(handlers::payments::list_lease_payments))
        .route("/payments/{id}", get(handlers::payments::get_payment))
        .route("/payments/{id}/receipt", get(handlers::payments::get_payment_receipt))
        .route("/payments/initiate", post(handlers::payments::initiate_payment))
        .route("/payments/history", get(handlers::payments::payment_history))
        // Escrow
        .route("/escrow/commit", post(handlers::escrow::commit_escrow))
        .route("/escrow/{id}", get(handlers::escrow::get_escrow))
        .route("/escrow/{id}/reveal", post(handlers::escrow::reveal_escrow))
        .route("/escrow/{id}/release", post(handlers::escrow::release_escrow))
        .route("/escrow/{id}/dispute", post(handlers::escrow::dispute_escrow))
        // Reputation
        .route("/reputation/me", get(handlers::reputation::get_my_reputation))
        .route("/reputation/me/proofs", get(handlers::reputation::get_my_proofs))
        .route("/reputation/me/credential", get(handlers::reputation::get_my_credential))
        .route("/reputation/{user_id}", get(handlers::reputation::get_user_reputation))
        // WebRTC
        .route("/webrtc/config", get(handlers::webrtc::get_webrtc_config))
}
