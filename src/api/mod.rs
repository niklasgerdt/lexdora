pub mod handlers;

use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use crate::api::handlers::*;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub fn build_router(pool: PgPool) -> Router {
    let state = AppState { pool };
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let api = Router::new()
        .route("/healthz", get(healthz))
        // organizations
        .route("/orgs", get(org::orgs_list).post(org::orgs_create))
        .route("/orgs/:id", get(org::orgs_get).patch(org::orgs_update).delete(org::orgs_delete))
        // users
        .route("/users", get(user::users_list).post(user::users_create))
        .route("/users/:id", get(user::users_get).patch(user::users_update).delete(user::users_delete))
        // incidents
        .route("/incidents", get(incident::incidents_list).post(incident::incidents_create))
        .route(
            "/incidents/:id",
            get(incident::incidents_get).patch(incident::incidents_update).delete(incident::incidents_delete),
        )
        // business units
        .route("/business-units", get(business_unit::business_units_list).post(business_unit::business_units_create))
        .route(
            "/business-units/:id",
            get(business_unit::business_units_get).patch(business_unit::business_units_update).delete(business_unit::business_units_delete),
        )
        // roles
        .route("/roles", get(role::roles_list).post(role::roles_create))
        .route(
            "/roles/:id",
            get(role::roles_get).patch(role::roles_update).delete(role::roles_delete),
        )
        .route(
            "/roles/:id/permissions",
            get(role::role_permissions_list).post(role::role_permissions_add),
        )
        .route(
            "/roles/:id/permissions/:pid",
            delete(role::role_permissions_remove),
        )
        // permissions
        .route("/permissions", get(permission::permissions_list).post(permission::permissions_create))
        .route(
            "/permissions/:id",
            get(permission::permissions_get).patch(permission::permissions_update).delete(permission::permissions_delete),
        )
        // user-role assignments
        .route(
            "/users/:id/roles",
            get(user::user_roles_list).post(user::user_roles_add),
        )
        .route(
            "/users/:id/roles/:rid",
            delete(user::user_roles_remove),
        )
        // third parties
        .route("/tpps", get(tpp::tpps_list).post(tpp::tpps_create))
        .route("/tpps/:id", get(tpp::tpps_get).patch(tpp::tpps_update).delete(tpp::tpps_delete))
        // ict assets
        .route("/assets", get(asset::assets_list).post(asset::assets_create))
        .route("/assets/:id", get(asset::assets_get).patch(asset::assets_update).delete(asset::assets_delete))
        .with_state(state.clone());

    // Static site
    let static_dir = ServeDir::new("web").not_found_service(ServeFile::new("web/index.html"));

    Router::new()
        .merge(api)
        .nest_service("/", static_dir)
        .layer(cors)
        .with_state(state)
}

pub async fn shutdown_signal() {
    use tokio::signal;
    let ctrl_c = async {
        signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {}, }
}
