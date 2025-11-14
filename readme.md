Zep is a minimal, super simple, async HTTP library in Rust.

It's designed to be minimal and easy to use, while still being very fast.

Only dependency is `tokio`.

# Example

This code returns a 200 OK response with the body `Hello world!` when GET / is requested.

```
use zep::{tokio, Router, Request, Response, Server, Method};

async fn root(_req: Request) -> Response {
    Response::ok("Hello world!")
}

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    router.route(Method::GET, "/", root);
    let server = Server::new("0.0.0.0:8080", router);
    let _ = server.run().await;
}
```

Check out our [docs](https://docs.rs/zep/latest/zep/)!