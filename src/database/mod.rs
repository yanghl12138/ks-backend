pub mod db;
pub mod query;
pub mod mutation;
pub mod search;

const DATADIR: &str = "data";

#[inline]
pub fn get_file_path(hash: &str) -> String {
    format!("{DATADIR}/{hash}")
}