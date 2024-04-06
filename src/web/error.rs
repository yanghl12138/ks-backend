use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;

use crate::Msg;

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
    UnsportFileType,

    // search
    NoSuchFile,
    ErrorSearchQuery,

    // user
    NoSuchUser,
    DuplicateUserName,
    InvalidPassword,
    EmptyUserName,
    UserHaveDocs,
    InvalidMoveUser,
    //
    TODO,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Error::LoginFail => "Login Fail",
            Error::InternalError => "Internal Error",
            Error::InvalidToken => "Invalid Token",
            Error::EmptyFileName => "Empty Filename",
            Error::UploadFail => "Uplord Fail",
            Error::DuplicateFile => "Duplicate File",
            Error::EmptyFile => "Empty File",
            Error::TODO => "To Do",
            Error::UnsportFileType => "UnsportFileType",
            Error::NoSuchFile => "No Such File",
            Error::ErrorSearchQuery => "Error Search Query",
            Error::NoSuchUser => "No Such User",
            Error::DuplicateUserName => "Duplicate UserName",
            Error::InvalidPassword => "Invalid Password",
            Error::EmptyUserName => "Empty UserName",
            Error::UserHaveDocs => "User Have Docs",
            Error::InvalidMoveUser => "Invalid Move User",
        };

        write!(f, "{}", output)
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("-->> {:<12} -- {self:?}", "INTO-RES");

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Msg {
                msg: self.to_string(),
            }),
        )
            .into_response()
    }
}

// 数据库类型的错误默认为InternalError
impl From<DbErr> for Error {
    fn from(_: DbErr) -> Self {
        Self::InternalError
    }
}

// io类型的错误默认为InternalError
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::InternalError
    }
}
