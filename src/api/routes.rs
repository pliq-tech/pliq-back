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
        .layer(axum_mw::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .nest("/api/v1", public)
        .nest("/api/v1", auth)
        .nest("/api/v1", protected)
        .with_state(state)
}
