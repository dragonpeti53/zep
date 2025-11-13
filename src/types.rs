use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Type alias of `HashMap<String, String>` for convenience.
pub type HeaderMap = HashMap<String, String>;
/// Type alias of `HashMap<String, String>` for convenience.
pub type ParamMap = HashMap<Arc<str>, Arc<str>>;

/// Enum for quick and memory efficient method handling.
#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    Other(String),
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            s => Method::Other(s.to_string()),
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
    /// Converts a `method` to its text representation in a &str type.
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

/// Enum for quick and memory efficient HTTP version handling.
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
    /// Converts a `Version` to its text representation in a &str type.
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

/// Enum to conveniently handle status codes.
#[derive(Debug, Clone, PartialEq)]
pub enum StatusCode {
    Ok,
    NotFound,
    InternalServerError,
    BadRequest,
    Forbidden,
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
    fn as_u16(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::NotFound => 404,
            StatusCode::InternalServerError => 500,
            StatusCode::BadRequest => 400,
            StatusCode::Forbidden => 403,
            StatusCode::Custom(c) => *c,
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::NotFound => "Not Found",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::Custom(_) => "Custom Code",
        }
    }
}

/// Deserialized HTTP request in the form of a struct for easy handling in code.
/// Contains request method, path, version, headers, body, remote_addr(ip address of client), and parameters.
pub struct Request {
    pub method: Method,
    pub path: Arc<str>,
    pub version: Version,
    pub headers: Arc<HeaderMap>,
    pub body: Arc<[u8]>,
    pub remote_addr: Arc<str>,
    pub params: Arc<ParamMap>,
}

/// Deserialized HTTP response in the form of a struct for easy handling in code.
/// Contains status_code(status code), headers and body.
#[derive(PartialEq, Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Response {
    /// Returns a new response.
    /// Requires a StatusCode, HeaderMap and generic body.
    pub fn new<B: Into<Vec<u8>>>(status_code: StatusCode, headers: HeaderMap, body: B) -> Self {
        Response {
            status_code,
            headers,
            body: body.into(),
        }
    }

    /// Helper function to conveniently return a 200 OK response with no headers.
    /// Requires generic body.
    pub fn ok<B: Into<Vec<u8>>>(body: B) -> Self {
        Response {
            status_code: StatusCode::Ok,
            headers: HeaderMap::new(),
            body: body.into(),
        }
    }

    /// Helper function to return a 404 Not Found response.
    pub fn not_found() -> Self {
        Response {
            status_code: StatusCode::NotFound,
            headers: HeaderMap::new(),
            body: b"404 Not Found".to_vec(),
        }
    }

    /// Helper function to return a 500 Internal Server Error response.
    pub fn error() -> Self {
        Response {
            status_code: StatusCode::InternalServerError,
            headers: HeaderMap::new(),
            body: "".into(),
        }
    }

    /// Appends a header to a response's headermap.
    /// Requires a key and value.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}
