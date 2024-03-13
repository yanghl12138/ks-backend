use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::request::Parts,
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::error::*;
use crate::{database::query::get_user_by_name, AppState};

// 定义jwt key
const JWT_SECRET: &str = "NJFU.EDU.CN";
pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}
impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
    pub fn global() -> Self {
        Self::new(JWT_SECRET.as_bytes())
    }
}

// 用户信息
#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    exp: usize,

    pub id: u64,
    pub username: String,
    pub is_admin: i8,
    pub level: u8,
}

// 登陆请求信息
#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

// 鉴权密钥
#[derive(Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: String::from("Bearer"),
        }
    }
}

// 从请求中提取和验证Claims
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Error::InvalidToken)?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &Keys::global().decoding,
            &Validation::default(),
        )
        .map_err(|_| Error::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub async fn login_api(
    state: State<AppState>,
    Json(paylord): Json<LoginPayload>,
) -> Result<Json<AuthBody>> {
    let user = match get_user_by_name(&state.conn, &paylord.username).await {
        Ok(user) => match user {
            Some(user) => user,
            None => {
                return Err(Error::LoginFail);
            }
        },
        Err(_) => {
            return Err(Error::InternalError);
        }
    };

    if user.password == paylord.password {
        let claims = Claims {
            exp: {
                let time: usize = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .try_into()
                    .unwrap();
                time + 60 * 60 * 8
            },

            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
            level: user.level,
        };
        let token = encode(&Header::default(), &claims, &Keys::global().encoding)
            .map_err(|_| Error::InternalError)?;

        Ok(Json(AuthBody::new(token)))
    } else {
        Err(Error::LoginFail)
    }
}

pub async fn whoami_api(claims: Claims) -> Result<Json<Claims>> {
    Ok(Json(claims))
}
