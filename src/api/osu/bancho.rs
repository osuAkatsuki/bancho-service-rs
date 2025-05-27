use crate::api::RequestContext;
use crate::events;
use crate::models::bancho::{BanchoRequest, BanchoResponse};

/// Controller for the osu! bancho protocol
pub async fn controller(ctx: RequestContext, request: BanchoRequest) -> BanchoResponse {
    events::handle_request(&ctx, request).await
}

pub async fn index() -> &'static str {
    "Running bancho-service v0.1"
}
