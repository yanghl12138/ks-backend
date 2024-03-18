use axum::extract::Path;
use axum::{extract::State, Json};
use serde::Deserialize;

use crate::database::mutation::add_user;
use crate::database::query::{get_all_users, get_user_by_id, get_user_by_name};
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
