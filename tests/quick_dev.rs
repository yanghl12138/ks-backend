#![allow(unused)]

use anyhow::Result;
use ks_backend::database::mutation::write_to_fs;
use ring::digest::{Context, SHA256};
use data_encoding::HEXUPPER;
#[tokio::test]

async fn quick_dev() -> Result<()> {
    write_to_fs("114514", b"1919810").await?;
    Ok(())
}