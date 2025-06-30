pub mod ripple;

use crate::common::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/isOnline", get(ripple::is_online))
        .route("/onlineUsers", get(ripple::online_users))
}
