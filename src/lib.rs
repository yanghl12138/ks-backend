use sea_orm::DatabaseConnection;
use serde::Serialize;

pub mod entities;
pub mod database;
pub mod web;

#[derive(Clone, Debug)]
pub struct AppState {
    pub conn: DatabaseConnection
}

#[derive(Serialize)]
pub struct Msg {
    pub msg: String
}

impl From<& str> for Msg{
    fn from(value: & str) -> Self {
        Self{ msg: value.to_string() }
    }
}