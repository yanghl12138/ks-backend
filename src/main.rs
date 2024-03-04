#[macro_use]
extern crate tantivy;
use axum::{
    extract::DefaultBodyLimit, routing::{get, post}, Json, Router
};
use ks_backend::{database::{db::*, search::{init_index, rebuild_search_index}}, web::login, AppState, Msg};
use tower_http::limit::RequestBodyLimitLayer;


#[tokio::main]
async fn main() {
    
    init_index();
    // initialize tracing
    tracing_subscriber::fmt::init();

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
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            50 * 1024 * 1024, /* 50mb */
        ))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let _ = jh.await.unwrap();

    println!("Listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Msg> {
    let msg: Msg = Msg::from("Hello World!");
    Json(msg)
}


