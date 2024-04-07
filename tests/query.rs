#![allow(unused)]


use core::time;
use std::thread::sleep;

use anyhow::Result;
use ks_backend::database::{db::get_db, mutation::write_file, query::read_file, search::{commiting, init_index, rebuild_search_index, search_from_rev_index, SearchField}};
use ring::digest::{Context, SHA256};
use data_encoding::HEXUPPER;




#[tokio::test]
async fn query_test() -> Result<()> {
    
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

    Ok(())
}