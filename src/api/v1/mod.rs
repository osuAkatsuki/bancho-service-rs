use crate::common::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
}
