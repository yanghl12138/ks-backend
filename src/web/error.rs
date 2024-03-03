use axum::{http::StatusCode, response::{IntoResponse, Response}};
use sea_orm::DbErr;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // login
    LoginFail,
    InternalError,
    InvalidToken,

    // uploard
    EmptyFileName,
    UploadFail,
    DuplicateFile,
    EmptyFile,

    //
    TODO,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("-->> {:<12} -- {self:?}", "INTO-RES");

        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_CLIENT_ERROR").into_response()
    }
    
}

// 数据库类型的错误默认为InternalError
impl From<DbErr> for Error {
    fn from(_: DbErr) -> Self {
        Self::InternalError
    }
}