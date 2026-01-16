use crate::message::StatusCode;

mod server;

mod client;

const STATUS_CODE_100_TRYING: StatusCode = StatusCode::Trying;
const STATUS_CODE_180_RINGING: StatusCode = StatusCode::Ringing;
const STATUS_CODE_202_ACCEPTED: StatusCode = StatusCode::Accepted;
const STATUS_CODE_301_MOVED_PERMANENTLY: StatusCode = StatusCode::MovedPermanently;
const STATUS_CODE_404_NOT_FOUND: StatusCode = StatusCode::NotFound;
const STATUS_CODE_504_SERVER_TIMEOUT: StatusCode = StatusCode::ServerTimeout;
const STATUS_CODE_603_DECLINE: StatusCode = StatusCode::Decline;
