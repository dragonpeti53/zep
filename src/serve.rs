use crate::Response;
use std::fs;

pub fn file(path: &str) -> Response {
    match fs::exists(path) {
        Ok(true) => 
            match fs::read(path) {
                Ok(contents) => Response::ok(contents),
                Err(_) => Response::error(),
            },
        Ok(false) => Response::not_found(),
        Err(_) => Response::not_found(),
    }
    
}