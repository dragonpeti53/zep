pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct Response {
    pub status_code: u16,
    pub reason: String,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: u16, reason: String, body: Vec<u8>) -> Self {
        Response {
            status_code,
            reason,
            body,
        }
    }

    pub fn ok(body: &str) -> Self {
        Response {
            status_code: 200,
            reason: "OK".to_string(),
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn not_found() -> Self {
        Response {
            status_code: 404,
            reason: "Not Found".to_string(),
            body: b"404 Not Found".to_vec(),
        }
    }
}