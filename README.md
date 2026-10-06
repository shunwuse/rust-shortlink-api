# rust-shortlink

A URL shortener with visit stats.

## Goals

- Shorten long URLs into shareable codes
- Redirect fast with a single binary + Docker image
- Track hits per link with a simple stats endpoint

## Features

1. `POST /links`: shorten a URL, returns a code
2. `GET /:id`: 302 redirect to the original URL, increments hits
3. `GET /links/:id/stats`: hits and creation time for a code
4. SQLite storage via `SQLx`, single file, zero setup
5. Release build + Dockerfile for deployment

Out of scope for now: custom codes, expiration, rate limiting, metrics, Postgres.

## API

```text
POST /links
Content-Type: application/json

{ "url": "https://example.com/very/long/path" }

201 Created
{ "id": "aB3x9Q", "url": "https://example.com/very/long/path" }
```

```text
GET /:id -> 302 Found
Location: <original url>
```

```text
GET /links/:id/stats

200 OK
{
  "id": "aB3x9Q",
  "url": "https://example.com/very/long/path",
  "hits": 42,
  "created_at": "2026-10-05T00:00:00Z"
}
```

Errors are JSON (`{ "error": "message" }`) with the matching status code (400 / 404 / 422).

## Stack

- Web: `axum` + `tokio`
- DB: `sqlx` + `sqlite`
- Errors: `thiserror` + `anyhow`
- Codes: random 7-char base62 strings, `code TEXT PRIMARY KEY`

## Milestones

1. Server runs with `GET /health` + SQLite wired up, `POST /links` persists
2. Redirect + hits counting + stats endpoint working end to end
3. Unified error responses, URL validation, Dockerfile

## Acceptance

- `cargo fmt --check` and `cargo clippy` clean
- All three endpoints pass a `curl` round trip
- `docker build` image serves traffic
