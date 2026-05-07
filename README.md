# mini_backend

A minimal HTTP backend written in Rust (edition 2024) using only the standard library — no external dependencies.

## Endpoints

| Method | Path | Response |
|--------|------|----------|
| `GET` | `/health` | `200 OK` with body `OK` |
| `GET` | `/echo/<message>` | `200 OK` with body `<message>` |
| `GET` | any other path | `404 NOT FOUND` |
| any | anything | `400 BAD REQUEST` if method is not GET |

## Usage

```bash
cargo run
```

Server listens on `http://127.0.0.1:8080`.

```bash
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/echo/hello
```

## Project Structure

```
src/
  main.rs   — TCP listener and connection handling
  routes.rs — Request routing logic
  utils.rs  — HTTP response helpers
```
