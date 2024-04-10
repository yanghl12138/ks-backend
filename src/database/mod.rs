use tokio::fs::create_dir;
use std::path::Path;

pub mod db;
pub mod query;
pub mod mutation;
pub mod search;

const DATADIR: &str = "data";

pub async fn init_datadir() -> anyhow::Result<()> {
    let dirpath = Path::new(DATADIR);
    println!("-->> {:<12} --- dirpath is {dirpath:?}", "INIT_DATADIR");
    if dirpath.exists() {
        if !dirpath.is_dir() {
            return Err(anyhow::Error::msg("please check dirpath"));
        }
    } else {
        println!("-->> {:<12} --- {dirpath:?} not exist", "INIT_DATADIR");
        println!("-->> {:<12} --- create dir {dirpath:?}", "INIT_DATADIR");
        create_dir(dirpath).await?;
    }
    Ok(())
}


#[inline]
pub fn get_file_path(hash: &str) -> String {
    format!("{DATADIR}/{hash}")
}