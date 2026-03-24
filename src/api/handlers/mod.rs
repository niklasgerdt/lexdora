pub mod org;
pub mod user;
pub mod incident;
pub mod tpp;
pub mod business_unit;
pub mod role;
pub mod permission;
pub mod asset;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::error;

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

pub fn handle_db_error(err: sqlx::Error) -> axum::response::Response {
    match err {
        sqlx::Error::RowNotFound => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"not found"}))).into_response()
        }
        sqlx::Error::Database(db) => {
            error!(?db, "DB error");
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": db.message()}))).into_response()
        }
        other => {
            error!(?other, "DB error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"internal error"}))).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use sqlx::Error;

    #[tokio::test]
    async fn test_handle_db_error_not_found() {
        let err = Error::RowNotFound;
        let res = handle_db_error(err);
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_handle_db_error_internal() {
        let err = Error::WorkerCrashed;
        let res = handle_db_error(err);
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
