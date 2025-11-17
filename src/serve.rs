//! This is a helper module that contains useful utilities to serve and receive different kinds of content over HTTP.

use crate::{Response, StreamReader, StreamWriter, StatusCode};
use tokio::fs;
use tokio::io::{AsyncWriteExt};
use std::io::Result;

/// Returns a 200 OK response with the contents of file located at `path`.
/// Returns a 500 Internal Server Error response if file could not be read or found.
/// Returns a 404 Not Found response if file at `path` does not exist.
#[deprecated(
    since = "0.3.0",
    note = "Use [`send_file`] instead"
)]
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

/// Serves a file in a Response. If file is bigger than 64 KiB, then it will be streamed.
pub async fn send_file(path: &str) -> Result<Response> {
    let n = 64 * 1024;
    match fs::read(path).await {
        Ok(data) => {
            if data.len() <= n {
                Ok(Response::ok(data))
            } else {
                Ok(Response::stream(StatusCode::Ok, StreamWriter::new(fs::File::open(path).await?)))
            }
        },
        Err(e) => Err(e),
    }
}

/// Saves an incoming stream to a file.
pub async fn save_streamed_file(
    mut reader: StreamReader,
    path: &str
) -> Result<()> {
    let mut file = tokio::fs::File::create(path).await?;

    loop {
        match reader.next_chunk().await {
            Ok(Some(chunk)) => file.write_all(&chunk).await?,
            Ok(None) => break,
            Err(e) => return Err(e),
        }
    }

    file.flush().await?;
    file.sync_all().await?;
    Ok(())
}
