use std::io::Error;

use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait, Set};
use tokio::{fs::remove_file, fs::File, io::AsyncWriteExt};

use crate::entities::{prelude::*, *};

use super::get_file_path;



pub async fn add_txt_info(
    conn: &DatabaseConnection,
    title: &str,
    hash: &str,
    user_id: &u64,
    level: &u8,
) -> Result<u64, DbErr> {
    let new_txt = txt::ActiveModel {
        title: ActiveValue::set(title.to_owned()),
        hash: ActiveValue::set(hash.to_owned()),
        user_id: ActiveValue::set(user_id.to_owned()),
        level: ActiveValue::set(level.to_owned()),
        ..Default::default()
    };
    let res = Txt::insert(new_txt).exec(conn).await?;
    Ok(res.last_insert_id)
}

pub async fn add_user(
    conn: &DatabaseConnection,
    username: &str,
    password: &str,
    level: u8,
    is_admin: bool
) -> Result<u64, DbErr> {
    let new_user = user::ActiveModel {
        username: ActiveValue::set(username.to_owned()),
        is_admin: ActiveValue::set(is_admin as i8),
        level: ActiveValue::set(level),
        password: ActiveValue::set(password.to_owned()),
        ..Default::default()
    };

    let res = User::insert(new_user).exec(conn).await?;
    Ok(res.last_insert_id)
}


pub async fn write_to_fs(hash: &str, data: &[u8]) -> Result<(), Error> {
    let mut f = File::create(get_file_path(hash)).await?;
    let _ = f.write_all(data).await?;
    _ = f.flush();
    Ok(())
}

pub async fn delete_file(hash: &str) -> Result<(), Error> {
    remove_file(get_file_path(hash)).await?;
    Ok(())
}

pub async fn update_doc_info(
    conn: &DatabaseConnection,
    doc: txt::Model,
    title: String,
    level: u8,
) -> Result<txt::Model, DbErr> {
    let mut doc: txt::ActiveModel = doc.into();
    doc.title = Set(title);
    doc.level = Set(level);

    Ok(doc.update(conn).await?)
}
