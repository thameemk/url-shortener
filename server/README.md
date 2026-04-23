# URL Shortener - API Server

A URL shortener backend built with Rust, Axum, SeaORM, and PostgreSQL.

## Features

- Shorten long URLs with auto-generated or custom short codes
- Update existing short URLs (long URL, short code, expiry)
- Redirect short codes to their original URLs with expiry support
- Click analytics — track total clicks, IP address, user agent, and referer per URL
- Paginated URL listing
- Interactive API docs via Scalar

## API Docs

![Scalar API Docs](../docs/readme/api-doc-scalar.png)

Visit `/docs` after starting the server to explore the interactive API documentation.

## Tech Stack

- **[Axum](https://github.com/tokio-rs/axum)** — HTTP framework
- **[SeaORM](https://www.sea-ql.org/SeaORM/)** — async ORM with PostgreSQL
- **[utoipa](https://github.com/juhaku/utoipa)** — OpenAPI spec generation
- **[Scalar](https://scalar.com)** — API docs UI

## Getting Started

### Prerequisites

- Rust (stable)
- PostgreSQL

### Setup

1. Clone the repository and create a `.env` file:

```env
DATABASE_URL=postgres://user:password@localhost:5432/url_shortener
```

2. Run the server:

```bash
cargo run
```

Migrations run automatically on startup. The server listens on `http://0.0.0.0:8000`.

## Notes

- Short URLs expire after **10 days** by default. Pass `expires_at` to override.
- Custom short codes must be unique. A `409 Conflict` is returned if the code is already taken.
- Expired short URLs return `410 Gone` on redirect.
- Click recording never blocks the redirect — failures are silently ignored.
