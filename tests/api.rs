use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use rust_shortlink_api::{db, models::AppState, routes};

async fn test_app() -> axum::Router {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::create_tables(&pool).await;
    routes::create_router(AppState { db_pool: pool })
}

fn json_request(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get_request(uri: String) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method("GET")
        .body(Body::empty())
        .unwrap()
}

async fn response_json(app: axum::Router, req: Request<Body>) -> (StatusCode, serde_json::Value) {
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn full_link_lifecycle() {
    let app = test_app().await;

    // 1. Shorten a URL
    let (status, body) = response_json(
        app.clone(),
        json_request("/links", r#"{"url":"https://example.com/a"}"#),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(id.len(), 7);

    // 2. Invalid URL is rejected
    let (status, _) = response_json(app.clone(), json_request("/links", r#"{"url":"nope"}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // 3. Follow the short link
    let res = app
        .clone()
        .oneshot(get_request(format!("/{id}")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FOUND);
    assert_eq!(res.headers()["location"], "https://example.com/a");

    // 4. Unknown code is 404
    let (status, _) = response_json(app.clone(), get_request("/ZZZZZZZ".to_string())).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // 5. Stats show one hit
    let (status, body) = response_json(app, get_request(format!("/links/{id}/stats"))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["hits"], 1);
    assert_eq!(body["url"], "https://example.com/a");
}
