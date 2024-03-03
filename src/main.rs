use axum::{
    routing::{get, post}, Json, Router
};
use ks_backend::{database::db::*, web::login, AppState, Msg};


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let conn = match get_db().await {
        Ok(db) => db,
        Err(err) => panic!("{}", err)
    };
    
    let state = AppState {conn};

    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(login::login_api))
        .route("/whoami", get(login::whoami_api))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Msg> {
    let msg = Msg::new("Hello World!");
    Json(msg)
}


