#![allow(unused)]


use core::time;
use std::thread::sleep;

use anyhow::Result;
use ks_backend::database::{db::get_db, mutation::{delete_file, write_file}, query::read_file, search::{commiting, init_index, rebuild_search_index, search_from_rev_index, SearchField}};
use ring::digest::{Context, SHA256};
use data_encoding::HEXUPPER;




#[tokio::test]
async fn query_test() -> Result<()> {
    
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

    Ok(())
}