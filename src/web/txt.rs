use std::str::from_utf8;

use axum::extract::{Multipart, State};
use axum::Json;
use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use tantivy::doc;

use super::error::*;
use super::login::Claims;
use crate::database::mutation::{add_txt_info, write_to_fs};
use crate::database::query::{get_txt_by_hash, get_txt_by_id};
use crate::database::search::{get_fields, get_writer};
use crate::{entities::txt, AppState};

async fn save_file(
    state: AppState,
    claims: Claims,
    filename: String,
    data: Vec<u8>,
    hash_value: String,
) -> Result<txt::Model> {
    println!("Saving {}", filename);
    // 空文件
    if data.len() == 0 {
        println!("Empty File!!!");
        return Err(Error::EmptyFile);
    }
    // 重复文件
    match get_txt_by_hash(&state.conn, &hash_value).await? {
        Some(_) => {
            println!("Duplicate File!!!");
            return Err(Error::DuplicateFile);},
        None => (),
    }

    let txt = from_utf8(&data)
    .map_err(|_| {println!("UnsportFileType"); Error::UnsportFileType})?;

    // 文件信息写入数据库
    let id: u64 = add_txt_info(
        &state.conn,
        &filename,
        &hash_value,
        &claims.id,
        &claims.level,
    )
    .await?;
    //  文件写入本地
    let _ = write_to_fs(&hash_value, &data)
        .await
        .map_err(|_| Error::InternalError)?;
    // 形成索引
    let writer = get_writer();
    let writer = writer.read().await;
    let fields = get_fields();
    let _ = writer.add_document(doc!(
        fields.id => id,
        fields.title => filename.clone(),
        fields.body => txt
    ));

    let new_txt_info: txt::Model = get_txt_by_id(&state.conn, id).await?.unwrap();

    println!("{} Saved", filename);
    Ok(new_txt_info)
}

pub async fn upload_api(
    State(state): State<AppState>,
    claims: Claims,
    mut multipart: Multipart,
) -> Result<Json<Vec<txt::Model>>> {
    let mut upload_success = Vec::<txt::Model>::with_capacity(16);
    let mut join_handlers = Vec::with_capacity(16);

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::UploadFail)?
    {
        let mut ctx = Context::new(&SHA256);
        let filename = match field.file_name() {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => return Err(Error::EmptyFileName),
        };
        println!("Receiving {}", filename);
        let mut data: Vec<u8> = Vec::with_capacity(1024);
        while let Some(bytes) = field.chunk().await.map_err(|_| Error::UploadFail)? {
            ctx.update(&bytes);
            data.extend(bytes);
        }
        let hash_value: String = HEXUPPER.encode(ctx.finish().as_ref());

        let f = save_file(state.clone(), claims.clone(), filename, data, hash_value);
        join_handlers.push(tokio::spawn(f));
    }
    for jh in join_handlers {
        let res = match jh.await {
            Ok(j) => j,
            Err(_) => continue,
        };
        let res = match res {
            Ok(j) => j,
            Err(_) => continue,
        };
        upload_success.push(res);
    }
    
    Ok(Json(upload_success))
}
