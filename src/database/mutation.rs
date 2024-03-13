use std::{io::Error};

use sea_orm::{ActiveValue, DatabaseConnection, DbErr, EntityTrait};
use tokio::{fs::File, fs::remove_file, io::AsyncWriteExt};

use crate::entities::{prelude::*, *};

const DATADIR: &str = "data";


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

#[inline]
fn get_file_path(hash: &str) -> String {
    format!("{DATADIR}/{hash}")
}

pub async fn write_to_fs(hash: &str, data: &[u8]) -> Result<(), Error>{
    let mut f = File::create(get_file_path(hash)).await?;
    let _ = f.write_all(data).await?;
    _ = f.flush();
    Ok(())
}

pub async fn delete_file(hash: &str) -> Result<(), Error> {
    remove_file(get_file_path(hash)).await?;
    Ok(())
}