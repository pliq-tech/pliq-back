use axum::middleware as axum_mw;
use axum::routing::{delete, get, post, put};
use axum::Router;
use crate::api::handlers;
use crate::api::middleware::auth::auth_middleware;
use crate::AppState;

pub fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/ready", get(handlers::health::ready));

    let auth = Router::new()
        .route("/auth/verify-world-id", post(handlers::auth::verify_world_id));

    let protected = Router::new()
        .route("/users/me", get(handlers::users::get_me).put(handlers::users::update_me))
        .route("/listings", post(handlers::listings::create_listing).get(handlers::listings::list_listings))
        .route("/listings/{id}", get(handlers::listings::get_listing).put(handlers::listings::update_listing).delete(handlers::listings::delete_listing))
        .route("/applications", post(handlers::applications::create_application).get(handlers::applications::list_applications))
        .route("/applications/{id}", get(handlers::applications::get_application))
        .route("/applications/{id}/status", put(handlers::applications::update_application_status))
        .route("/leases", post(handlers::leases::create_lease).get(handlers::leases::list_leases))
        .route("/leases/{id}", get(handlers::leases::get_lease))
        .route("/leases/{id}/sign", put(handlers::leases::sign_lease))
        .route("/payments", post(handlers::payments::create_payment).get(handlers::payments::list_payments))
        .route("/payments/{id}", get(handlers::payments::get_payment))
        .route("/escrow/commit", post(handlers::escrow::commit_escrow))
        .route("/escrow/{id}/reveal", post(handlers::escrow::reveal_escrow))
        .route("/escrow/{id}/release", post(handlers::escrow::release_escrow))
        .route("/reputation/me", get(handlers::reputation::get_my_reputation))
        .route("/reputation/me/proofs", get(handlers::reputation::get_my_proofs))
        .layer(axum_mw::from_fn_with_state(state.clone(), auth_middleware));

    let public_reputation = Router::new()
        .route("/reputation/root", get(handlers::reputation::get_merkle_root));

    Router::new()
        .nest("/api/v1", public)
        .nest("/api/v1", auth)
        .nest("/api/v1", protected)
        .nest("/api/v1", public_reputation)
        .with_state(state)
}
