#![allow(unused)]

use core::time;
use std::thread::sleep;

use anyhow::Result;
use data_encoding::HEXUPPER;
use httpc_test::*;
use ks_backend::{
    database::{
        db::get_db,
        mutation::write_file,
        query::get_txt_by_id,
        search::{commiting, init_index, rebuild_search_index, search_from_rev_index, SearchField},
    },
    entities::txt,
};
use ring::digest::{Context, SHA256};
use sea_orm::DatabaseConnection;

async fn search_test(
    conn: &DatabaseConnection,
    field: SearchField,
    q: &str,
    level: u8,
    limit: usize,
) -> anyhow::Result<Vec<txt::Model>> {
    let res = search_from_rev_index(field, "红色 燃烧", level, limit)?;
    let mut docs = Vec::new();

    for (id, _) in res {
        let doc = get_txt_by_id(conn, id).await?;
        match doc {
            Some(doc) => docs.push(doc),
            None => (),
        }
    }

    Ok(docs)
}

#[tokio::test]
async fn search() -> Result<()> {
    init_index();

    println!("Connect to DataBase...");
    let conn: sea_orm::prelude::DatabaseConnection = match get_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };
    println!("DataBase Connected ...");

    rebuild_search_index(conn.clone()).await;
    tokio::spawn(commiting());
    sleep(time::Duration::from_secs(5));

    println!("搜索功能测试");
    println!("---");
    match search_test(&conn, SearchField::All, "+红色 -街道", 255, 2).await {
        Ok(res) => {
            for e in res {
                println!("{e:?}");
            }
        }
        Err(e) => println!("{e:?}"),
    }
    println!("---");
    match search_test(&conn, SearchField::Body, "路西恩", 255, 0).await {
        Ok(res) => {
            for e in res {
                println!("{e:?}");
            }
        }
        Err(e) => println!("{e:?}"),
    }
    println!("---");
    match search_test(&conn, SearchField::Body, "路西*", 64, 0).await {
        Ok(res) => {
            for e in res {
                println!("{e:?}");
            }
        }
        Err(e) => println!("{e:?}"),
    }

    println!("---");
    match search_test(&conn, SearchField::Body, "*", 64, 0).await {
        Ok(res) => {
            for e in res {
                println!("{e:?}");
            }
        }
        Err(e) => println!("{e:?}"),
    }

    Ok(())
}
