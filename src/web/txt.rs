use std::str::from_utf8;

use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};
use sea_orm::ModelTrait;
use serde::Deserialize;
use tantivy::doc;

use super::error::*;
use super::login::Claims;
use crate::database::mutation::{add_txt_info, delete_file, write_to_fs};
use crate::database::query::{get_txt_by_hash, get_txt_by_id, get_txt_by_user_id};
use crate::database::search::{delete_from_index, get_fields, get_writer, search_from_rev_index, SearchField};
use crate::Msg;
use crate::{entities::txt, AppState};

/// 保存文件
async fn save_file(
    state: AppState,
    claims: Claims,
    filename: String,
    data: Vec<u8>,
    hash_value: String,
) -> Result<txt::Model> {
    println!("-->> {:<12} -- Saving {filename:?}", "SAVE_FILE");
    // 空文件
    if data.len() == 0 {
        println!("-->> {:<12} -- EMPTY_FILE", "SAVE_FILE");
        return Err(Error::EmptyFile);
    }
    // 重复文件
    match get_txt_by_hash(&state.conn, &hash_value).await? {
        Some(_) => {
            println!("-->> {:<12} -- Duplicate File", "SAVE_FILE");
            return Err(Error::DuplicateFile);
        }
        None => (),
    }

    let txt = from_utf8(&data).map_err(|_| {
        println!("-->> {:<12} -- UnSupportFileType", "SAVE_FILE");
        Error::UnsportFileType
    })?;

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
    let doc = doc!(
        fields.id => id,
        fields.title => filename.clone(),
        fields.body => txt,
        fields.level => claims.level as u64
    );
    let _ = writer.add_document(doc);

    let new_txt_info: txt::Model = get_txt_by_id(&state.conn, id).await?.unwrap();

    println!("-->> {:<12} -- {filename:?} Saved", "SAVE_FILE");
    Ok(new_txt_info)
}

/// 文件上传，接受multipartform
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

/// 查看用户拥有的所有文档
pub async fn docs_info_api(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<txt::Model>>> {
    let res = get_txt_by_user_id(&state.conn, claims.id).await?;
    Ok(Json(res))
}

/// 查看可以查看的文档
pub async fn doc_info_api(
    State(state): State<AppState>,
    claims: Claims,
    Path(doc_id): Path<u64>,
) -> Result<Json<txt::Model>> {
    let doc = get_txt_by_id(&state.conn, doc_id).await?;
    match doc {
        Some(doc) if doc.level <= claims.level => Ok(Json(doc)),
        _ => Err(Error::NoSuchFile),
    }
}

/// 删除文档
pub async fn delete_doc_api(
    State(state): State<AppState>,
    claims: Claims,
    Path(doc_id): Path<u64>,
) -> Result<Json<Msg>> {
    let doc = get_txt_by_id(&state.conn, doc_id).await?;
    let doc = match doc {
        Some(doc) if claims.id == doc.user_id => doc,
        _ => return Err(Error::NoSuchFile),
    };

    // 从索引中删除
    let _ = delete_from_index(doc.id).await;
    // 从文件系统中删除
    let _ = delete_file(&doc.hash).await;
    // 从数据库中删除
    let _ = doc.delete(&state.conn).await;

    Ok(Json(Msg::from("Ok")))
}

#[derive(Deserialize, Clone)]
pub struct QueryArg {
    query_string: String,
    field: String,
    limit: usize
}

/// 查询api
pub async fn query_api(
    State(state): State<AppState>,
    claims: Claims,
    query_arg: Query<QueryArg>
) -> Result<Json<Vec<txt::Model>>> {
    let query_string = &query_arg.query_string;
    let limit = query_arg.limit.to_owned();
    let field = SearchField::from(query_arg.field.to_owned());

    let res_id = search_from_rev_index(field, query_string, claims.level, limit)
    .map_err(|_| Error::ErrorSearchQuery)?;

    let mut res = Vec::new();
    for id in res_id {
        println!("-->>{:<12} -- {id}", "QUERY_API");
        match get_txt_by_id(&state.conn, id).await? {
            Some(doc) => res.push(doc),
            None => (),
        }
    }
    Ok(Json(res))
}