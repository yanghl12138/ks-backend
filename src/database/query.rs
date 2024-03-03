use super::super::entities::{prelude::*, *};
use sea_orm::*;

pub async fn get_user_by_id(conn: &DatabaseConnection, id: u64) -> Result<Option<user::Model>, DbErr> {
    User::find_by_id(id).one(conn).await
}

pub async fn get_user_by_name(conn: &DatabaseConnection, username: &str) -> Result<Option<user::Model>, DbErr> {
    User::find().filter(user::Column::Username.eq(username)).one(conn).await
}

pub async fn get_txt_by_id(conn: &DatabaseConnection, id: u64) -> Result<Option<txt::Model>, DbErr> {
    Txt::find_by_id(id).one(conn).await
}

pub async fn get_txt_by_hash(conn: &DatabaseConnection, hash: &str) -> Result<Option<txt::Model>, DbErr> {
    Txt::find().filter(txt::Column::Hash.eq(hash)).one(conn).await
}
