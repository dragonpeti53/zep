//! This is a helper module for convenience to easily serve different types of content over HTTP.

use crate::Response;
use tokio::fs;

/// Returns a 200 OK response with the contents of file located at `path`.
/// Returns a 500 Internal Server Error response if file could not be read or found.
/// Returns a 404 Not Found response if file at `path` does not exist.
pub async fn file(path: &str) -> Response {
    match fs::try_exists(path).await {
        Ok(true) => match fs::read(path).await {
            Ok(contents) => Response::ok(contents),
            Err(_) => Response::error(),
        },
        Ok(false) => Response::not_found(),
        Err(_) => Response::error(),
    }
}
