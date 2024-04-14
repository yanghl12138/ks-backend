#![allow(unused)]


use core::time;
use std::thread::sleep;

use anyhow::Result;
use ks_backend::database::{db::get_db, mutation::write_file, query::{get_txt_maxlevel_by_userid, read_file}, search::{commiting, init_index, rebuild_search_index, search_from_rev_index, SearchField}};
use ring::digest::{Context, SHA256};
use data_encoding::HEXUPPER;

use urlencoding::{encode, decode};


#[tokio::test]
async fn query_test() -> Result<()> {
    println!("Connect to DataBase...");
    let conn: sea_orm::prelude::DatabaseConnection = match get_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    println!("DataBase Connected ...");
    // 文件读取测试
    let hash = "F19FA689A97AD895398684F0861FB267DFE785EDC52FF465F2999C484A9DFC55".to_string();
    let res = read_file(hash).await;
    if res.is_ok() {
        println!("Ok");
    } else {
        println!("Err")
    }

    let hash = "F19FA89A97AD895398684F0861FB267DFE785EDC52FF465F2999C484A9DFC55".to_string();
    let res = read_file(hash).await;
    if res.is_ok() {
        println!("Ok");
    } else {
        println!("Err")
    }
    //
    let res = get_txt_maxlevel_by_userid(&conn, 3).await.unwrap();
    println!("{res:?}");

    //
    let sc = "你好";
    let se = "Hello";
    let sce = encode(sc);
    println!("{sce}");
    let sced = decode("(%E4%BD%A0%E5%A5%BD)").unwrap();
    println!("{sced}");

    Ok(())
}