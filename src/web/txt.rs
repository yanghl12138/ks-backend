use axum::body::Bytes;
use axum::extract::{Multipart, State};
use ring::digest::{Context, Digest, SHA256};
use data_encoding::HEXUPPER;
use tokio::join;

use crate::database::mutation::{add_txt_info, write_to_fs};
use crate::database::query::{get_txt_by_hash, get_txt_by_id};
use crate::{entities::txt, AppState};
use super::error::*;
use super::login::Claims;

pub async fn uploard_api(state: State<AppState>, claims: Claims, mut multipart: Multipart) 
-> Result<txt::Model> {
    // 读取form
    let mut filename: String = String::new();
    let mut data: Vec<u8> = Vec::with_capacity(1024);

    // sha256
    let mut ctx = Context::new(&SHA256);

    if let Some(mut field) = multipart.next_field().await.map_err(|_| Error::UploadFail)? {
        filename = match field.file_name() {
            Some(n) => n.to_string(),
            None => return Err(Error::EmptyFileName)
        };
        while let Some(bytes) = field.chunk().await.map_err(|_| Error::UploadFail)? {
            ctx.update(&bytes);
            data.extend(bytes);
        }
    }
    // 非空
    if data.len() == 0 {
        return Err(Error::EmptyFile);
    }
    let hash_value: String = HEXUPPER.encode(ctx.finish().as_ref());
    // 查重
    match
    get_txt_by_hash(&state.conn, &hash_value).await? {
        Some(_) => return Err(Error::DuplicateFile),
        None => (),
    }
    
    // 文件信息写入数据库
    let id = add_txt_info(&state.conn, &filename, &hash_value, &claims.id, &claims.level)
    .await?;
    // 形成倒排索引
    todo!();
    // 文件写入本地
    let _ = write_to_fs(&hash_value, &data).await.map_err(|_| {
        Error::InternalError
    })?;
    
    let new_txt_info = get_txt_by_id(&state.conn, id).await?.unwrap();

    Ok(new_txt_info)
}