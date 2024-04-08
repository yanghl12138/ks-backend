#![allow(unused)]


use core::time;
use std::thread::sleep;

use anyhow::Result;
use ks_backend::database::{db::get_db, mutation::{add_txt_info, add_user, delete_file, delete_txt_info, delete_user, write_file}, query::{get_txt_by_id, get_user_by_id, read_file}, search::{commiting, init_index, rebuild_search_index, search_from_rev_index, SearchField}};
use ring::digest::{Context, SHA256};
use data_encoding::HEXUPPER;




#[tokio::test]
async fn query_test() -> Result<()> {
    println!("Connect to DataBase...");
    let conn: sea_orm::prelude::DatabaseConnection = match get_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    println!("DataBase Connected ...");
    
    // 文件写入测试
    
    let hash = "114514";
    let data = "1919810".as_bytes();

    if write_file(hash, data).await.is_ok() {
        println!("Ok");
    } else {
        println!("Err");
    }
    
    if delete_file(hash).await.is_ok() {
        println!("Ok");
    } else {
        println!("Err");
    }

    // 用户增删测试

    let id = add_user(&conn, "test", "test", 0, false).await.expect("Add User Failure");
    println!("Add user Ok");
    // 添加一个用户
    let user = get_user_by_id(&conn, id).await.unwrap().unwrap();
    // 添加一个文档
    let doc_id = add_txt_info(&conn, "123", "123", &id, &0).await.unwrap();
    // 尝试删除，应该失败    
    if delete_user(&conn, user.clone(), None).await.is_ok() {
        panic!("Test Has Doc");
    } else {
        println!("Ok")
    }
    // 删除该文档
    let doc = get_txt_by_id(&conn, doc_id).await.unwrap().unwrap();
    delete_txt_info(&conn, doc).await.unwrap();
    // 再次删除，应该成功
    delete_user(&conn, user, None).await.expect("Delete User Failure");
    
    println!("Delete User Ok");


    Ok(())
}