use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct CreateLink {
    url: String,
}

#[derive(Serialize)]
struct Link {
    id: String,
    url: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Serialize)]
struct Stats {
    id: String,
    url: String,
    hits: i64,
    created_at: String,
}

#[derive(Clone)]
struct AppState {
    db_pool: SqlitePool,
}

const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn encode_base62(mut num: i64) -> String {
    let mut encoded = Vec::new();

    if num == 0 {
        return "0".to_string();
    }

    while num > 0 {
        let rem = (num % 62) as usize;
        encoded.push(BASE62_CHARS[rem]);
        num /= 62;
    }

    encoded.reverse();

    String::from_utf8(encoded).unwrap()
}

fn decode_base62(s: &str) -> Option<i64> {
    let mut num: i64 = 0;

    for b in s.bytes() {
        let v = BASE62_CHARS.iter().position(|&c| c == b)? as i64;
        num = num.checked_mul(62)?.checked_add(v)?;
    }

    Some(num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base62() {
        assert_eq!(encode_base62(0), "0");
        assert_eq!(encode_base62(1), "1");
        assert_eq!(encode_base62(61), "z");
        assert_eq!(encode_base62(62), "10");
        assert_eq!(encode_base62(3843), "zz");
        assert_eq!(encode_base62(238327), "zzz");
    }

    #[test]
    fn test_decode_base62() {
        assert_eq!(decode_base62("0"), Some(0));
        assert_eq!(decode_base62("1"), Some(1));
        assert_eq!(decode_base62("z"), Some(61));
        assert_eq!(decode_base62("10"), Some(62));
        assert_eq!(decode_base62("zz"), Some(3843));
        assert_eq!(decode_base62("zzz"), Some(238327));
        assert_eq!(decode_base62("nope-!"), None);
        assert_eq!(decode_base62(""), Some(0));
    }
}

async fn create_link(
    State(state): State<AppState>,
    Json(payload): Json<CreateLink>,
) -> Result<(StatusCode, Json<Link>), (StatusCode, Json<ErrorBody>)> {
    if !payload.url.starts_with("http://") && !payload.url.starts_with("https://") {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorBody {
                error: "url must start with http:// or https://".to_string(),
            }),
        ));
    }

    let result = sqlx::query("INSERT INTO links (url, hits) VALUES (?, 0)")
        .bind(&payload.url)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: e.to_string(),
                }),
            )
        })?;

    let id = encode_base62(result.last_insert_rowid());

    Ok((
        StatusCode::CREATED,
        Json(Link {
            id,
            url: payload.url,
        }),
    ))
}

async fn redirect_link(Path(id): Path<String>) -> impl IntoResponse {
    let _ = id;
    (
        StatusCode::FOUND,
        [(header::LOCATION, "https://example.com")],
    )
}

async fn link_stats(Path(id): Path<String>) -> Json<Stats> {
    Json(Stats {
        id,
        url: "https://example.com".to_string(),
        hits: 0,
        created_at: "2026-10-05T00:00:00Z".to_string(),
    })
}

#[tokio::main]
async fn main() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite:shortlink.db?mode=rwc")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            hits INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let state = AppState { db_pool: pool };
    let app = Router::new()
        .route("/health", get(health))
        .route("/links", post(create_link))
        .route("/links/{id}/stats", get(link_stats))
        .route("/{id}", get(redirect_link))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
