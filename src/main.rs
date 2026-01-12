use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use nanoid::nanoid;
use serde::Deserialize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    //Initialize the Tracing-subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let state = AppState {
        db: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(handler))
        .route("/shorten", post(shorten))
        .route("/{id}", get(redirect_url))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 7878));
    tracing::info!("Listening On {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, Rust World!"
}

async fn shorten(State(state): State<AppState>, Json(payload): Json<CreateRequest>) -> String {
    let id = nanoid!(6);

    let mut db = state.db.lock().unwrap();

    db.insert(id.clone(), payload.url);

    format!("http://localhost:7878/{}", id)
}

async fn redirect_url(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();

    if let Some(url) = db.get(&id) {
        Redirect::to(url).into_response()
    } else {
        (StatusCode::NOT_FOUND, "ID not found").into_response()
    }
}

#[derive(Deserialize)]
struct CreateRequest {
    url: String,
}

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<HashMap<String, String>>>,
}
