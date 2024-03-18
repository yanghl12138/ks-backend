use crate::{
    database::mutation::add_user,
    entities::{prelude::*, *},
};
use data_encoding::HEXUPPER;
use dotenv::dotenv;
use ring::digest::{Context, SHA256};
use sea_orm::*;
use std::env;

const DATABASE_DEFAULT_URL: &str = "mysql://root:123456@127.0.0.1:3306/ksdb";

fn find_database_url_from_env() -> Option<String> {
    dotenv().ok();
    for (key, value) in env::vars() {
        if key == "DATABASE_URL" {
            return Some(value);
        }
    }
    None
}

fn find_admin_from_env() -> (String, String) {
    dotenv().ok();
    let mut username: Option<String> = None;
    let mut password_beare: Option<String> = None;
    for (key, value) in env::vars() {
        if key == "ADMIN_USERNAME" {
            username = Some(value);
        } else if key == "ADMIN_PASSWORD_BEARE" {
            password_beare = Some(value);
        }
    }
    let username = username.expect("SET ADMIN_USERNAME!!!");
    let password_beare = password_beare.expect("SET ADMIN_PASSWORD_BEARE!!!");
    (username, password_beare)
}

pub async fn get_db() -> Result<DatabaseConnection, DbErr> {
    let database_url = match find_database_url_from_env() {
        Some(url) => url,
        None => DATABASE_DEFAULT_URL.to_owned(),
    };
    println!("-->>{:<12} --- DataBase url: {database_url}", "GETDB");
    let db = Database::connect(database_url).await?;
    Ok(db)
}

pub async fn init_admin_user(conn: DatabaseConnection) {
    // 存在admin?
    println!("-->>{:<12} --- INIT ADNIM", "INIT_ADMIN_USER");
    let admin = User::find()
        .filter(user::Column::IsAdmin.eq(1 as i8))
        .one(&conn)
        .await
        .unwrap();
    if admin.is_some() {
        // 存在，则什么都不做
        println!("-->>{:<12} --- ADNIM EXIST", "INIT_ADMIN_USER");
        ()
    } else {
        // 不存在，根据环境变量初始化admin
        println!("-->>{:<12} --- ADNIM NOT EXIST", "INIT_ADMIN_USER");
        println!("-->>{:<12} --- INIT ADNIM FROM DOTENV", "INIT_ADMIN_USER");
        let (username, password_beare) = find_admin_from_env();
        // 密码加密
        let mut ctx = Context::new(&SHA256);
        ctx.update(password_beare.as_bytes());
        let password: String = HEXUPPER.encode(ctx.finish().as_ref());
        // 添加用户
        add_user(&conn, &username, &password, 255, true)
            .await
            .unwrap();
        println!("-->>{:<12} --- ADD ADMIN {username}", "INIT_ADMIN_USER");
    }
}
