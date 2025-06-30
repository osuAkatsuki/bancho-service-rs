use crate::common::error::ServiceResult;
use crate::common::state::AppState;
use redis::Msg;

pub async fn handle(_ctx: AppState, _msg: Msg) -> ServiceResult<()> {
    unimplemented!()
}
