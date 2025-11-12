use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    Other(String)
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            s => Method::Other(s.to_string())
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
            Method::Other(_) => write!(f, "OTHER"),
        }
    }
}

impl Method {
    pub fn to_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::Other(_) => "OTHER",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    Http10,
    Http11,
    Http2,
    Http3,
    Other,
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        match s {
            "HTTP/1.0" => Version::Http10,
            "HTTP/1.1" => Version::Http11,
            "HTTP/2.0" | "HTTP/2" => Version::Http2,
            "HTTP/3.0" | "HTTP/3" => Version::Http3,
            _ => Version::Other,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Http10 => write!(f, "HTTP/1.0"),
            Version::Http11 => write!(f, "HTTP/1.1"),
            Version::Http2 => write!(f, "HTTP/2.0"),
            Version::Http3 => write!(f, "HTTP/3.0"),
            Version::Other => write!(f, "OTHER"),
        }
    }
}

impl Version {
    pub fn to_str(&self) -> &str {
        match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
            Version::Http2 => "HTTP/2.0",
            Version::Http3 => "HTTP/3.0",
            Version::Other => "OTHER",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusCode {
    Ok,
    NotFound,
    InternalServerError,
    Custom(u16),
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.as_u16();
        let reason = self.reason();
        write!(f, "{} {}", code, reason)
    }
}

impl StatusCode {
    pub fn as_u16(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::NotFound => 404,
            StatusCode::InternalServerError => 500,
            StatusCode::Custom(c) => *c,
        }
    }

    pub fn reason(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::NotFound => "Not Found",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::Custom(_) => "Custom Code",
        }
    }
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: Version,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub remote_addr: String,
}

#[derive(PartialEq, Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status_code: StatusCode, body: Vec<u8>) -> Self {
        Response {
            status_code,
            body,
        }
    }

    pub fn ok(body: &str) -> Self {
        Response {
            status_code: StatusCode::Ok,
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn not_found() -> Self {
        Response {
            status_code: StatusCode::NotFound,
            body: b"404 Not Found".to_vec(),
        }
    }

    pub fn error() -> Self {
        Response {
            status_code: StatusCode::InternalServerError,
            body: "".into(),
        }
    }
}