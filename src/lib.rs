//! # Zep is an async HTTP crate focusing on minimalism and simplicity.
//! I made it out of frustration with mainstream HTTP crates, I found that they were too bloated for simple use cases.
//! I just wanted something simple, minimal, and reasonably fast.
//!
//! # Example
//!
//! Return a 200 OK response with content "Hello world!" at "GET / ".
//!
//! ```no_run
//! use zep::{tokio, Router, Request, Response, Server, Method};
//!
//! async fn root(_req: Request) -> Response {
//!     Response::ok("Hello world!")
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut router = Router::new();
//!     router.route(Method::GET, "/", root);
//!
//!     let server = Server::new("0.0.0.0:8080", router);
//!     let _ = server.run().await;
//! }
//! ```
//!
//!

mod route;
pub mod serve;
mod server;
mod tests;
mod types;

pub use route::{Handler, Router};
pub use server::{Server, StreamReader, StreamWriter};
/// Re-exporting tokio for user convenience.
pub use tokio;
pub use types::{HeaderMap, Method, ParamMap, Request, Response, StatusCode, Version};
//pub use serve;
