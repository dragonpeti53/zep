//! This is a helper module for convenience to easily serve different types of content over HTTP.

use crate::{Response, StreamReader};
use tokio::fs;
use tokio::io::{BufReader, AsyncReadExt, AsyncBufReadExt, AsyncWriteExt};
use std::io;

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

pub async fn save_streamed_file(
    //reader: &mut StreamReader,
    mut reader: StreamReader,
    path: &str
) -> std::io::Result<()> {
    //let mut buf_reader = BufReader::new(reader);
    let mut buf_reader = BufReader::new(&mut reader);
    let mut file = tokio::fs::File::create(path).await?;

    loop {
        let mut size_line = String::new();
        let n = buf_reader.read_line(&mut size_line).await?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected eof while reading chunk size",
            ));
        }

        let size_hex = size_line
            .trim_end_matches(&['\r', '\n'][..])
            .split(';')
            .next()
            .unwrap_or("0")
            .trim();

        let size = match usize::from_str_radix(size_hex, 16) {
            Ok(s) => s,
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid chunk size: {}", e),
                ))
            }
        };

        if size == 0 {
            loop {
                let mut trailer = String::new();
                let n = buf_reader.read_line(&mut trailer).await?;
                if n == 0 || trailer == "\r\n" || trailer.trim().is_empty() {
                    break;
                }
            }
            break;
        }

        let mut remaining = size;
        let mut buffer = vec![0u8; 8 * 1024];
        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buffer.len());
            buf_reader.read_exact(&mut buffer[..to_read]).await?;
            file.write_all(&buffer[..to_read]).await?;
            remaining -= to_read;
        }

        let mut crlf = [0u8; 2];
        buf_reader.read_exact(&mut crlf).await?;
    }

    file.flush().await?;
    file.sync_all().await?;
    Ok(())
}
