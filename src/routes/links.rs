use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::codes::{gen_code, is_unique_violation};
use crate::error::AppError;
use crate::models::{AppState, CreateLink, Link, Stats};

pub async fn create_link(
    State(state): State<AppState>,
    Json(payload): Json<CreateLink>,
) -> Result<(StatusCode, Json<Link>), AppError> {
    if !payload.url.starts_with("http://") && !payload.url.starts_with("https://") {
        return Err(AppError::InvalidUrl);
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
            Err(e) => return Err(e.into()),
        }
    }
}

pub async fn redirect_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT url FROM links WHERE code = ?")
        .bind(&id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(AppError::Db)?;
    let Some((url,)) = row else {
        return Err(AppError::NotFound);
    };

    sqlx::query("UPDATE links SET hits = hits + 1 WHERE code = ?")
        .bind(&id)
        .execute(&state.db_pool)
        .await
        .ok();

    let location = HeaderValue::from_str(&url).unwrap_or(HeaderValue::from_static("/"));

    Ok((StatusCode::FOUND, [(header::LOCATION, location)]).into_response())
}

pub async fn link_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Stats>, AppError> {
    let row: Option<(String, i64, String)> =
        sqlx::query_as("SELECT url, hits, created_at FROM links WHERE code = ?")
            .bind(&id)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(AppError::Db)?;

    let Some((url, hits, created_at)) = row else {
        return Err(AppError::NotFound);
    };

    Ok(Json(Stats {
        id,
        url,
        hits,
        created_at,
    }))
}
