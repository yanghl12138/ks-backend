extern crate tantivy;
use axum::{
    extract::DefaultBodyLimit, routing::{get, post}, Json, Router
};
use ks_backend::{database::{db::*, search::{commiting, init_index, rebuild_search_index}}, web::{login, txt}, AppState, Msg};
use tokio::time::{self, sleep};
use tower_http::limit::RequestBodyLimitLayer;


#[tokio::main]
async fn main() {
    
    init_index();

    println!("Connect to DataBase...");
    let conn: sea_orm::prelude::DatabaseConnection = match get_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err)
    };
    println!("DataBase Connected ...");


    let jh = tokio::spawn(rebuild_search_index(conn.clone()));

    let state = AppState {conn};

    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(login::login_api))
        .route("/whoami", get(login::whoami_api))
        .route("/doc", post(txt::upload_api).get(txt::docs_info_api))
        .route("/doc/:id", get(txt::doc_info_api).delete(txt::delete_doc_api))
        .route("/query", get(txt::query_api))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            50 * 1024 * 1024, /* 50mb */
        ))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let _ = jh.await.unwrap();
    tokio::spawn(commiting());
    let _ = sleep(time::Duration::from_millis(5000));
    println!("Listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Msg> {
    let msg: Msg = Msg::from("Hello World!");
    Json(msg)
}


