use super::{
    super::entities::{prelude::*, *},
    get_file_path,
};
use sea_orm::*;
use std::io::Error;
use tokio::{fs::File, io::AsyncReadExt};

pub async fn get_user_by_id(
    conn: &DatabaseConnection,
    id: u64,
) -> Result<Option<user::Model>, DbErr> {
    User::find_by_id(id).one(conn).await
}

pub async fn get_user_by_name(
    conn: &DatabaseConnection,
    username: &str,
) -> Result<Option<user::Model>, DbErr> {
    User::find()
        .filter(user::Column::Username.eq(username))
        .one(conn)
        .await
}

pub async fn get_all_users(conn: &DatabaseConnection) -> Result<Vec<user::Model>, DbErr> {
    User::find().all(conn).await
}

pub async fn get_txt_by_id(
    conn: &DatabaseConnection,
    id: u64,
) -> Result<Option<txt::Model>, DbErr> {
    Txt::find_by_id(id).one(conn).await
}

pub async fn get_txt_by_user_id(
    conn: &DatabaseConnection,
    user_id: u64,
) -> Result<Vec<txt::Model>, DbErr> {
    Txt::find()
        .filter(txt::Column::UserId.eq(user_id))
        .all(conn)
        .await
}

pub async fn get_txt_by_hash(
    conn: &DatabaseConnection,
    hash: &str,
) -> Result<Option<txt::Model>, DbErr> {
    Txt::find()
        .filter(txt::Column::Hash.eq(hash))
        .one(conn)
        .await
}

pub async fn get_all_txt(conn: &DatabaseConnection) -> Result<Vec<txt::Model>, DbErr> {
    Txt::find().all(conn).await
}

pub async fn get_all_txt_lte_level(
    conn: &DatabaseConnection,
    level: u8,
) -> Result<Vec<txt::Model>, DbErr> {
    Txt::find()
        .filter(txt::Column::Level.lte(level))
        .all(conn)
        .await
}

/// 根据user_id找到txt中最大的level
pub async fn get_txt_maxlevel_by_userid(
    conn: &DatabaseConnection,
    user_id: u64,
) -> Result<Option<u8>, DbErr> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        max_level: Option<u8>,
    }
    let res = Txt::find()
        .select_only()
        .column_as(txt::Column::Level.max(), "max_level")
        .filter(txt::Column::UserId.eq(user_id))
        .into_model::<SelectResult>()
        .one(conn)
        .await?;
    let res = if let Some(res) = res {
        if let Some(v) = res.max_level {
            Some(v)
        } else {
            None
        }
    } else {
        None
    };
    Ok(res)
}

pub async fn read_file(hash: String) -> Result<String, Error> {
    let mut f = match File::open(get_file_path(&hash)).await {
        Ok(f) => f,
        Err(e) => {
            println!("-->>{:<12} --- {hash:?} may not exsist!!!", "READ_FROM_FS");
            return Err(e);
        }
    };
    let mut buf = String::with_capacity(15000);
    let _ = match f.read_to_string(&mut buf).await {
        Ok(_) => (),
        Err(e) => {
            println!("-->>{:<12} --- {hash:?} not UTF-8", "READ_FROM_FS");
            return Err(e);
        }
    };
    Ok(buf)
}
