use axum::extract::{Path, Query};
use axum::{extract::State, Json};
use serde::Deserialize;

use crate::database::mutation::{add_user, delete_user, update_user_info};
use crate::database::query::{get_all_users, get_txt_by_user_id, get_user_by_id, get_user_by_name};
use crate::Msg;
use crate::{entities::user, AppState};

use super::error::*;
use super::login::Claims;

/// 验证是否为admin
#[inline]
fn validate_admin(claims: &Claims) -> Result<()> {
    if claims.is_admin == 0 {
        return Err(Error::InvalidToken);
    }
    Ok(())
}

// 所有用户信息
pub async fn users_info_api(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<user::Model>>> {
    let _ = validate_admin(&claims)?;

    let users = get_all_users(&state.conn).await?;
    Ok(Json(users))
}

// 用户信息
pub async fn user_info_api(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<u64>,
) -> Result<Json<user::Model>> {
    let _ = validate_admin(&claims)?;

    let user = get_user_by_id(&state.conn, id).await?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err(Error::NoSuchUser),
    }
}

const EMPTY_PASSWORD: &str = "EC3395932920B3DA7BE44CFE7673CA15EA24DE40214905CEE00EE274F8C1CE6F";
#[derive(Deserialize, Debug, Clone)]
pub struct NewUser {
    username: String,
    // use sha256
    password: String,
    level: u8,
}

// 添加用户
pub async fn add_user_info_api(
    State(state): State<AppState>,
    claims: Claims,
    Json(paylord): Json<NewUser>,
) -> Result<Json<user::Model>> {
    let _ = validate_admin(&claims)?;
    // 验证用户名唯一
    match get_user_by_name(&state.conn, &paylord.username).await? {
        Some(_) => return Err(Error::DuplicateUserName),
        None => (),
    }
    // 验证密码，sha256，非空
    let password_sha256 = paylord.password.to_ascii_uppercase();
    if password_sha256.len() != 64 || password_sha256 == EMPTY_PASSWORD {
        return Err(Error::InvalidPassword);
    }
    //
    let user_id = add_user(
        &state.conn,
        &paylord.username,
        &password_sha256,
        paylord.level,
        false,
    )
    .await?;
    let new_user = get_user_by_id(&state.conn, user_id)
        .await?
        .ok_or(Error::InternalError)?;

    Ok(Json(new_user))
}

#[derive(Clone, Deserialize)]
pub struct UpdateUserInfo {
    username: String,
    level: u8,
    password: String,
}

pub async fn update_user_info_api(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateUserInfo>,
) -> Result<Json<user::Model>> {
    let _ = validate_admin(&claims)?;
    // 检验用户名
    // 非空
    if payload.username.is_empty() {
        return Err(Error::EmptyUserName);
    }
    // 非重复
    if let Some(u) = get_user_by_name(&state.conn, &payload.username).await? {
        if u.id != id {
            return Err(Error::DuplicateUserName);
        }
    }
    // 检验密码
    let password_sha256 = payload.password.to_ascii_uppercase();
    if password_sha256.len() != 64 || password_sha256 == EMPTY_PASSWORD {
        return Err(Error::InvalidPassword);
    }
    // 修改用户信息
    let user = get_user_by_id(&state.conn, id)
        .await?
        .ok_or(Error::NoSuchUser)?;
    let user = update_user_info(
        &state.conn,
        user,
        payload.username,
        payload.level,
        password_sha256,
    )
    .await?;
    Ok(Json(user))
}


#[derive(Deserialize, Clone)]
pub struct DeleteUserArg {
    to: Option<u64>,
}
pub async fn delete_user_api(
    State(state): State<AppState>,
    claims: Claims,
    Query(delete_user_arg): Query<DeleteUserArg>,
     Path(id): Path<u64>
) -> Result<Json<Msg>> {
    let _ = validate_admin(&claims)?;
    let okmsg = Json(Msg::from("Ok"));
    // 不允许自己删除自己
    if id == claims.id {
        return Err(Error::NotAllowDeleteYourSelf)
    }
    // 待删除的用户
    let user = get_user_by_id(&state.conn, id)
        .await?
        .ok_or(Error::NoSuchUser)?;
    // 未提供to
    if delete_user_arg.to.is_none() {
        // 检测是否有文档
        if get_txt_by_user_id(&state.conn, id).await?.is_empty() {
            delete_user(&state.conn, user, None).await?;
            Ok(okmsg)
        } else {
            Err(Error::UserHaveDocs)
        }
    // 提供to
    } else {
        // 获取moveuser
        let moveuser = get_user_by_id(&state.conn, delete_user_arg.to.unwrap())
            .await?
            .ok_or(Error::TODO)?;
        if user.level > moveuser.level || user.id == moveuser.id {
            Err(Error::InvalidMoveUser)
        // moveuser.level必须大于等于user.level，且moveuser和user不能是同一个
        } else {
            delete_user(&state.conn, user, Some(moveuser)).await?;
            Ok(okmsg)
        }
    }
}