# rust-shortlink-api

A URL shortener with visit stats.

## Goals

- Shorten long URLs into shareable codes
- Redirect fast with a single binary + Docker image
- Track hits per link with a simple stats endpoint

## Stack

- Web: `axum` + `tokio`
- DB: `sqlx` + `sqlite`
- Errors: unified `AppError` mapped to status codes in one place
- Codes: random 7-char base62 strings, `code TEXT PRIMARY KEY`

## Project Layout

```text
src/main.rs      # bootstrap: config, pool, routes, serve
src/config.rs    # PORT and DATABASE_URL from env
src/handlers.rs  # HTTP handlers (no SQL here besides store calls)
src/db.rs        # pool setup and table creation
src/codes.rs     # short code generation
src/models.rs    # request/response structs and app state
src/error.rs     # AppError and status code mapping
```

## Quick Start

Local run (requires Rust):

```bash
cargo run
```

The server listens on `localhost:3000` and creates `shortlink.db` on first start.

Docker (no Rust needed):

```bash
docker build -t rust-shortlink-api .
docker run -d --rm -p 3000:3000 --name shortlink rust-shortlink-api
curl localhost:3000/health
docker stop shortlink
```

## Demo Script

Copy-paste the whole block:

```bash
# 1. Shorten a URL (save the returned id)
curl -X POST localhost:3000/links \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com/very/long/path"}'
# -> {"id":"fOGMy6E","url":"https://example.com/very/long/path"}

# 2. Follow the short link (replace the id)
curl -i localhost:3000/fOGMy6E
# -> HTTP/1.1 302 Found, Location: https://example.com/very/long/path

# 3. Check stats (hits is now 1)
curl localhost:3000/links/fOGMy6E/stats
# -> {"id":"fOGMy6E","url":"https://example.com/very/long/path","hits":1,...}

# 4. Unknown code
curl -i localhost:3000/ZZZZZZZ
# -> 404 {"error":"link not found"}

# 5. Invalid URL
curl -X POST localhost:3000/links \
  -H 'Content-Type: application/json' \
  -d '{"url":"not-a-url"}'
# -> 422 {"error":"url must start with http:// or https://"}
```

Note: codes are random 7-char strings, so ids in your run will differ.
Quote URLs containing `!` (e.g. `'localhost:3000/nope-!!'`) — bare `!`
triggers bash history expansion.

## Configuration

| Variable     | Default                                      | Purpose                    |
| ------------ | -------------------------------------------- | -------------------------- |
| `PORT`       | `3000`                                       | Port to listen on          |
| `DATABASE_URL` | `sqlite:shortlink.db?mode=rwc`               | SQLite database file       |
| `RUST_LOG`   | `rust_shortlink_api=debug,tower_http=debug`  | Log filter (tracing syntax) |

```bash
PORT=8080 cargo run
# serves on localhost:8080

RUST_LOG=warn cargo run
# quiets request logs
```

Invalid `PORT` values fall back to `3000`.

## API

```text
POST /links
Content-Type: application/json

{ "url": "https://example.com/very/long/path" }

201 Created
{ "id": "fOGMy6E", "url": "https://example.com/very/long/path" }
```

```text
GET /:id -> 302 Found
Location: <original url>
```

```text
GET /links/:id/stats

200 OK
{
  "id": "fOGMy6E",
  "url": "https://example.com/very/long/path",
  "hits": 42,
  "created_at": "2026-10-06 09:13:02"
}
```

Errors are JSON (`{ "error": "message" }`) with the matching status code
(404 / 422 / 500).

## Development

```bash
cargo test                  # unit tests
cargo fmt --check           # formatting
cargo clippy -- -D warnings # lints
```

Out of scope for now: custom codes, expiration, rate limiting, metrics, Postgres.
