extern crate tantivy;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Json, Router,
};
use ks_backend::{
    database::{
        db::*,
        search::{commiting, init_index, rebuild_search_index},
    },
    web::{
        login,
        txt::{self, download_api},
        user,
    },
    AppState, Msg,
};
use tokio::time::{self, sleep};
use tower_http::limit::RequestBodyLimitLayer;

#[tokio::main]
async fn main() {
    // 初始化索引
    init_index();

    // 获取数据库
    println!("-->{:<12} --- Connect to DataBase...", "MAIN");
    let conn: sea_orm::prelude::DatabaseConnection = get_db().await.expect(&format!(
        "-->{:<12} --- Can Not Connect to DataBase",
        "MAIN"
    ));
    println!("-->{:<12} --- DataBase Connected", "MAIN");

    // 初始化admin
    let jh_admin_index = tokio::spawn(init_admin_user(conn.clone()));
    // 建立索引
    let jh_build_index = tokio::spawn(rebuild_search_index(conn.clone()));

    let state = AppState { conn };

    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(login::login_api))
        .route("/whoami", get(login::whoami_api))
        .route("/doc", post(txt::upload_api).get(txt::docs_info_api))
        .route(
            "/doc/:id",
            get(txt::doc_info_api)
                .delete(txt::delete_doc_api)
                .put(txt::update_doc_api),
        )
        .route("/download/:hash", get(download_api))
        .route("/query/:hash", get(txt::doc_info_hash_api))
        .route("/query", get(txt::query_api))
        .route("/index", post(txt::rebuild_index_api))
        .route(
            "/user",
            get(user::users_info_api).post(user::add_user_info_api),
        )
        .route(
            "/user/:id",
            get(user::user_info_api).put(user::update_user_info_api),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(50 * 1024 * 1024 /* 50mb */))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let _ = jh_build_index.await.unwrap();
    let _ = jh_admin_index.await.unwrap();
    tokio::spawn(commiting());
    let _ = sleep(time::Duration::from_millis(5000));
    println!("Listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Msg> {
    let msg: Msg = Msg::from("Hello World!");
    Json(msg)
}
