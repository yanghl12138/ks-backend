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

pub async fn get_txt_by_user_id(conn: &DatabaseConnection, user_id: u64) -> Result<Vec<txt::Model>, DbErr> {
    Txt::find().filter(txt::Column::UserId.eq(user_id)).all(conn).await
}

pub async fn get_txt_by_hash(conn: &DatabaseConnection, hash: &str) -> Result<Option<txt::Model>, DbErr> {
    Txt::find().filter(txt::Column::Hash.eq(hash)).one(conn).await
}

pub async fn get_all_txt(conn: &DatabaseConnection) -> Result<Vec<txt::Model>, DbErr> {
    Txt::find().all(conn).await
}
