#![allow(unused)]


use core::time;
use std::thread::sleep;

use anyhow::Result;
use ks_backend::database::{db::get_db, mutation::write_to_fs, search::{commiting, init_index, rebuild_search_index, search_from_rev_index, SearchField}};
use ring::digest::{Context, SHA256};
use data_encoding::HEXUPPER;
use httpc_test::*;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    init_index();

    println!("Connect to DataBase...");
    let conn: sea_orm::prelude::DatabaseConnection = match get_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err)
    };
    println!("DataBase Connected ...");

    // let jh = tokio::spawn(rebuild_search_index(conn.clone()));
    // let _ = jh.await.unwrap();
    rebuild_search_index(conn.clone()).await;
    tokio::spawn(commiting());
    sleep(time::Duration::from_secs(5));
    println!("---");
    let res = search_from_rev_index(SearchField::Title, "红色 燃烧", 129, 6).unwrap();
    for e in res {
        println!("{e}")
    }
    println!("---");
    let res = search_from_rev_index(SearchField::Body, "路西恩", 129, 0).unwrap();
    for e in res {
        println!("{e}")
    }
    
    Ok(())
}