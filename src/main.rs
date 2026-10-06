use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rand::RngExt;
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
const CODE_LEN: usize = 7;

fn gen_code() -> String {
    let mut rng = rand::rng();

    (0..CODE_LEN)
        .map(|_| {
            let idx = rng.random_range(0..BASE62_CHARS.len());
            BASE62_CHARS[idx] as char
        })
        .collect()
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.code())
        .is_some_and(|code| code == "2067")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_codes_have_expected_shape() {
        for _ in 0..100 {
            let code = gen_code();
            assert_eq!(code.len(), CODE_LEN);
            assert!(code.bytes().all(|b| BASE62_CHARS.contains(&b)));
        }
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

    loop {
        let code = gen_code();
        match sqlx::query("INSERT INTO links (code, url, hits) VALUES (?, ?, 0)")
            .bind(&code)
            .bind(&payload.url)
            .execute(&state.db_pool)
            .await
        {
            Ok(_) => {
                return Ok((
                    StatusCode::CREATED,
                    Json(Link {
                        id: code,
                        url: payload.url,
                    }),
                ));
            }
            Err(e) if is_unique_violation(&e) => continue,
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: e.to_string(),
                    }),
                ));
            }
        }
    }
}

async fn redirect_link(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let row: Option<(String,)> = sqlx::query_as("SELECT url FROM links WHERE code = ?")
        .bind(&id)
        .fetch_optional(&state.db_pool)
        .await
        .unwrap_or(None);
    let Some((url,)) = row else {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: "link not found".to_string(),
            }),
        )
            .into_response();
    };

    sqlx::query("UPDATE links SET hits = hits + 1 WHERE code = ?")
        .bind(&id)
        .execute(&state.db_pool)
        .await
        .ok();

    let location = HeaderValue::from_str(&url).unwrap_or(HeaderValue::from_static("/"));

    (StatusCode::FOUND, [(header::LOCATION, location)]).into_response()
}

async fn link_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Stats>, (StatusCode, Json<ErrorBody>)> {
    let row: Option<(String, i64, String)> =
        sqlx::query_as("SELECT url, hits, created_at FROM links WHERE code = ?")
            .bind(&id)
            .fetch_optional(&state.db_pool)
            .await
            .unwrap_or(None);

    let Some((url, hits, created_at)) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorBody {
                error: "link not found".to_string(),
            }),
        ));
    };

    Ok(Json(Stats {
        id,
        url,
        hits,
        created_at,
    }))
}

#[tokio::main]
async fn main() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite:shortlink.db?mode=rwc")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (
            code TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            hits INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
