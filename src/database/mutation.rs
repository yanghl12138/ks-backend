use std::io::Error;

use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter, Set
};
use tokio::{fs::remove_file, fs::File, io::AsyncWriteExt};

use crate::entities::{prelude::*, *};

use super::{get_file_path, query::get_txt_by_user_id};

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

pub async fn delete_txt_info(
    conn: &DatabaseConnection,
    txt: txt::Model
) -> Result<(), DbErr> {
    txt.delete(conn).await?;
    Ok(())
}

pub async fn add_user(
    conn: &DatabaseConnection,
    username: &str,
    password: &str,
    level: u8,
    is_admin: bool,
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

pub async fn write_file(hash: &str, data: &[u8]) -> Result<(), Error> {
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
    title: Option<String>,
    level: Option<u8>,
) -> Result<txt::Model, DbErr> {
    let mut doc: txt::ActiveModel = doc.into();
    if let Some(title) = title {
        doc.title = Set(title);
    }
    if let Some(level) = level {
        doc.level = Set(level);
    }

    Ok(doc.update(conn).await?)
}

pub async fn update_user_info(
    conn: &DatabaseConnection,
    user: user::Model,
    username: Option<String>,
    level: Option<u8>,
    password: Option<String>,
) -> Result<user::Model, DbErr> {
    let mut user: user::ActiveModel = user.into();
    if let Some(username) = username {
        user.username = Set(username);
    }
    if let Some(level) = level {
        user.level = Set(level);
    }
    if let Some(password) = password {
        user.password = Set(password);
    }
    Ok(user.update(conn).await?)
}

/// 将一个用户的所有文档转移到另一个用户
async fn move_onwer(
    conn: &DatabaseConnection,
    from: user::Model,
    to: user::Model,
) -> Result<(), DbErr> {
    if from.id == to.id {
        return Ok(());
    }
    if from.level > to.level {
        return Err(DbErr::RecordNotUpdated);
    }
    let _ = Txt::update_many()
        .col_expr(txt::Column::UserId, Expr::value(to.id))
        .filter(txt::Column::UserId.eq(from.id))
        .exec(conn)
        .await?;
    Ok(())
}

/// 删除某个用户
pub async fn delete_user(
    conn: &DatabaseConnection,
    user: user::Model,
    move_to: Option<user::Model>,
) -> Result<(), DbErr> {
    let docs = get_txt_by_user_id(conn, user.id).await?;
    // 没有文档直接删除
    if docs.is_empty() {
        user.delete(conn).await?;
        Ok(())
    // 有文档，提供move_to，尝试转移文档所有权，再删除
    } else if move_to.is_some() {
        let move_to = move_to.unwrap();
        move_onwer(conn, user.clone(), move_to).await?;
        user.delete(conn).await?;
        Ok(())
    } else {
        Err(DbErr::RecordNotUpdated)
    }
}
