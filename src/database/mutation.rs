use std::{fs::File, io::{Error, Write} };

use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait};

use crate::entities::{prelude::*, *};

pub async fn add_txt_info
(conn: &DatabaseConnection, title: &str, hash: &str, user_id: &u64, level: &u8)
-> Result<u64, DbErr> {
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

pub async fn write_to_fs(hash: &str, data: &[u8]) -> Result<(), Error>{
    let mut f = File::create(format!("data/{}", hash))?;
    let _ = f.write_all(data)?;
    _ = f.flush();
    Ok(())
}