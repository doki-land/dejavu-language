# axum-dejavu

Axum integration for Dejavu. Package name uses the **axum-** host prefix.

```rust
use axum::{routing::get, Router};
use axum_dejavu::{context_from_iter, html_from_source};
use serde_json::json;

async fn hello() -> impl axum::response::IntoResponse {
    html_from_source(
        "Hello, <% name %>!",
        &context_from_iter([("name", json!("World"))]),
    )
}

fn app() -> Router {
    Router::new().route("/", get(hello))
}
```

IR-first (cross-language identical output):

```rust
use axum_dejavu::html_from_ir_json;
```

Does not change Dejavu IR semantics; only adapts `render` results to Axum `Html`.
