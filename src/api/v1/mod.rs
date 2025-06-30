pub mod ripple;

use crate::common::state::AppState;
use axum::Router;
use axum::routing::get;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/isOnline", get(ripple::is_online))
        .route("/onlineUsers", get(ripple::online_users))
        .route("/serverStatus", get(ripple::server_status))
        .route("/verifiedStatus", get(ripple::verified_status))
        .route("/playerMatchDetails", get(ripple::player_match_details))
}
