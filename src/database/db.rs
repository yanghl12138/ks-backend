use sea_orm::*;
use dotenv::dotenv;
use std::env;

const DATABASE_DEFAULT_URL: & str = "mysql://root:123456@127.0.0.1:3306/ksdb";

fn find_database_url_from_env() -> Option<String> {
    dotenv().ok();
    for (key, value) in env::vars() {
        if key == "DATABASE_URL" {
            return Some(value);
        }
    }
    None
}

pub async fn get_db() -> Result<DatabaseConnection, DbErr> {
    let database_url = match find_database_url_from_env() {
        Some(url) => url,
        None => DATABASE_DEFAULT_URL.to_owned()
    };
    let db = Database::connect(database_url).await?;
    Ok(db)
}