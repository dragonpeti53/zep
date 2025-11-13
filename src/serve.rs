//! This is a helper module for convenience to easily serve different types of content over HTTP.

use crate::Response;
use std::fs;

/// Returns a 200 OK response with the contents of file located at `path`.
/// Returns a 500 Internal Server Error response if file could not be read.
/// Returns a 404 Not Found response if file at `path` does not exist or could not be found.
pub fn file(path: &str) -> Response {
    match fs::exists(path) {
        Ok(true) => match fs::read(path) {
            Ok(contents) => Response::ok(contents),
            Err(_) => Response::error(),
        },
        Ok(false) => Response::not_found(),
        Err(_) => Response::not_found(),
    }
}
