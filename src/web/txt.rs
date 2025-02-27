use std::cmp::min;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, HeaderMap};
use axum::Json;
use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};

use serde::{Deserialize, Serialize};
use tantivy::doc;

use urlencoding::{decode, encode};

use super::error::*;
use super::login::Claims;
use crate::database::mutation::{
    add_txt_info, delete_file, delete_txt_info, update_doc_info, write_file,
};
use crate::database::query::{get_all_txt_lte_level, get_txt_by_hash, get_txt_by_id, read_file};
use crate::database::search::{
    add_doc_to_index, delete_from_index, rebuild_search_index, search_from_rev_index, SearchField,
};
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
    // 转化为UTF-8并验证
    let txt = String::from_utf8(data).map_err(|_| {
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
    let _ = write_file(&hash_value, txt.as_bytes())
        .await
        .map_err(|_| Error::InternalError)?;
    // 形成索引
    add_doc_to_index(id, filename.clone(), txt, claims.level).await;

    // 返回信息
    let new_txt_info: txt::Model = get_txt_by_id(&state.conn, id)
        .await?
        .ok_or(Error::InternalError)?;
    println!("-->> {:<12} -- {filename:?} Saved", "SAVE_FILE");
    Ok(new_txt_info)
}

/// 单文件上传，接受multipartform，成功返回文件信息，失败返回错误信息
pub async fn upload_doc_api(
    State(state): State<AppState>,
    claims: Claims,
    mut multipart: Multipart,
) -> Result<Json<txt::Model>> {
    if let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::UploadFail)?
    {
        let mut ctx = Context::new(&SHA256);
        let filename = match field.name() {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => match field.file_name() {
                Some(n) if !n.is_empty() => n.to_string(),
                _ => return Err(Error::EmptyFileName),
            },
        };
        println!("Receiving {}", filename);
        let mut data: Vec<u8> = Vec::with_capacity(1024);
        while let Some(bytes) = field.chunk().await.map_err(|_| Error::UploadFail)? {
            ctx.update(&bytes);
            data.extend(bytes);
        }
        let hash_value: String = HEXUPPER.encode(ctx.finish().as_ref());
        let doc = save_file(state, claims, filename, data, hash_value).await?;
        Ok(Json(doc))
    } else {
        Err(Error::EmptyFile)
    }
}

/// 多文件上传，接受multipartform，返回上传成功的文件信息
pub async fn upload_docs_api(
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
        let filename = match field.name() {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => match field.file_name() {
                Some(n) if !n.is_empty() => n.to_string(),
                _ => continue,
            },
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

/// 查看用户可见所有文档
pub async fn docs_info_api(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<txt::Model>>> {
    let res = get_all_txt_lte_level(&state.conn, claims.level).await?;
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
/// 根据hash查看文档信息
pub async fn doc_info_hash_api(
    State(state): State<AppState>,
    claims: Claims,
    Path(hash): Path<String>,
) -> Result<Json<txt::Model>> {
    let hash = hash.to_ascii_uppercase();
    let doc = get_txt_by_hash(&state.conn, &hash).await?;
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
    let _ = delete_txt_info(&state.conn, doc).await;

    Ok(Json(Msg::from("Ok")))
}

#[derive(Deserialize, Clone)]
pub struct QueryArg {
    query_string: String,
    field: Option<String>,
    limit: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct QueryResult {
    doc: txt::Model,
    score: f32,
}

impl QueryResult {
    pub fn new(doc: txt::Model, score: f32) -> QueryResult {
        QueryResult { doc, score }
    }
}

/// 查询api
pub async fn query_api(
    State(state): State<AppState>,
    claims: Claims,
    query_arg: Query<QueryArg>,
) -> Result<Json<Vec<QueryResult>>> {
    let query_string = decode(&query_arg.query_string).map_err(|_| Error::ErrorSearchQuery)?;
    let limit = query_arg.limit.to_owned();
    let field = SearchField::from(query_arg.field.to_owned().unwrap_or("All".to_string()));

    let res_id = search_from_rev_index(field, &query_string, claims.level, limit.unwrap_or(0))
        .map_err(|_| Error::ErrorSearchQuery)?;

    let mut res = Vec::new();
    for (id, score) in res_id {
        // println!("-->>{:<12} -- {id}", "QUERY_API");
        match get_txt_by_id(&state.conn, id).await? {
            Some(doc) => res.push(QueryResult::new(doc, score)),
            None => (),
        }
    }
    Ok(Json(res))
}

#[derive(Clone, Deserialize)]
pub struct UpdateDocInfo {
    title: Option<String>,
    level: Option<u8>,
}
/// 更新文档信息
pub async fn update_doc_api(
    State(state): State<AppState>,
    claims: Claims,
    Path(doc_id): Path<u64>,
    Json(payload): Json<UpdateDocInfo>,
) -> Result<Json<txt::Model>> {
    let doc = get_txt_by_id(&state.conn, doc_id).await?;
    let doc = match doc {
        // 验证权限
        Some(doc) if doc.user_id == claims.id => doc,
        _ => return Err(Error::NoSuchFile),
    };
    let title = {
        // 字符串非空，否则为原title
        if let Some(tit) = payload.title {
            if tit.is_empty() {
                Some(doc.title.clone())
            } else {
                Some(tit)
            }
        } else {
            None
        }
    };

    // level不超过用户level，否则为用户level
    let level = {
        if let Some(lev) = payload.level {
            let level = min(claims.level, lev);
            Some(level)
        } else {
            None
        }
    };

    // 如果没有改变，直接返回
    if level.is_none() && title.is_none() {
        return Ok(Json(doc));
    }
    if let Some(level) = level {
        if let Some(title) = &title {
            if level == doc.level && doc.title == *title {
                return Ok(Json(doc));
            }
        }
    }

    // 修改数据库
    let doc = update_doc_info(&state.conn, doc, title, level).await?;
    // 修改索引
    let _ = delete_from_index(doc.id).await;
    let body = read_file(doc.hash.clone()).await?;
    add_doc_to_index(doc.id, doc.title.clone(), body, doc.level).await;
    Ok(Json(doc))
}

/// 重建索引，需要admin
pub async fn rebuild_index_api(State(state): State<AppState>, claims: Claims) -> Result<Json<Msg>> {
    if claims.is_admin == 1 {
        rebuild_search_index(state.conn).await;
        Ok(Json(Msg::from("Ok")))
    } else {
        Err(Error::InvalidToken)
    }
}

/// 下载文件
pub async fn download_api(
    State(state): State<AppState>,
    claims: Claims,
    Path(hash): Path<String>,
) -> Result<(HeaderMap, String)> {
    let hash = hash.to_ascii_uppercase();
    // 获取文件信息
    let doc = match get_txt_by_hash(&state.conn, &hash).await? {
        // 鉴权
        Some(doc) if doc.level <= claims.level => Ok(doc),
        _ => Err(Error::NoSuchFile),
    }?;

    // 设置头
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/plain; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", encode(&doc.title))
            .parse()
            .unwrap(),
    );
    let body = read_file(hash).await?;

    Ok((headers, body))
}
