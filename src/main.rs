use axum::{http::StatusCode, routing::{get, post}, Json, Router};
use rand::seq::SliceRandom;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct GameState {
    board: Board,
    you: Snake,
}

#[derive(Deserialize)]
struct Board {
    width: i32,
    height: i32,
}

#[derive(Deserialize)]
struct Snake {
    body: Vec<Coord>,
}

#[derive(Deserialize, Clone, Copy, PartialEq)]
struct Coord {
    x: i32,
    y: i32,
}

// GET / — Battlesnake's "ping": snake metadata and appearance.
async fn info() -> Json<Value> {
    Json(json!({
        "apiversion": "1",
        "author": "bingo",
        "color": "#e8590c",
        "head": "default",
        "tail": "default",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn ok() -> StatusCode {
    StatusCode::OK
}

// POST /move — random direction that doesn't run into our own body or a wall.
async fn mv(Json(state): Json<GameState>) -> Json<Value> {
    let body = &state.you.body;
    let head = body[0];

    // The tail moves out of the way next turn, unless it's stacked (just ate / game start).
    let n = body.len();
    let tail_moves = n >= 2 && body[n - 1] != body[n - 2];
    let blocked = if tail_moves { &body[..n - 1] } else { &body[..] };

    let safe: Vec<&str> = [("up", 0, 1), ("down", 0, -1), ("left", -1, 0), ("right", 1, 0)]
        .into_iter()
        .filter(|&(_, dx, dy)| {
            let next = Coord { x: head.x + dx, y: head.y + dy };
            let on_board = (0..state.board.width).contains(&next.x)
                && (0..state.board.height).contains(&next.y);
            on_board && !blocked.contains(&next)
        })
        .map(|(name, _, _)| name)
        .collect();

    let choice = safe.choose(&mut rand::thread_rng()).copied().unwrap_or("up");
    Json(json!({ "move": choice }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(info))
        .route("/start", post(ok))
        .route("/move", post(mv))
        .route("/end", post(ok));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    println!("listening on :{port}");
    axum::serve(listener, app).await.unwrap();
}
