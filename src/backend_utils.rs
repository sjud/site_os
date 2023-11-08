use super::*;
use std::fmt::Debug;
pub fn handle_server_err(err: impl Debug, user_msg: String) -> ServerFnErrorErr {
    log::info!("{:?}", err);
    ServerFnErrorErr::ServerError(user_msg)
}

/* 
pub fn time_now() -> time::PrimitiveDateTime {
    let now = time::OffsetDateTime::now_utc();
    time::PrimitiveDateTime::new(now.date(), now.time())
}*/