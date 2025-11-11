type Handler = fn(Request) -> Response;
type Logger = fn(&Request);
type _Middleware = fn(&Request);

use crate::types::{Request, Response};

#[derive(Clone)]
struct Route {
    method: String,
    path: String,
    handler: Handler,
}

#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
    logger: Option<Logger>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            logger: None,
        }
    }

    pub fn route(&mut self, method: &str, path: &str, handler: Handler) {
        self.routes.push(
            Route {
                method: method.to_string(),
                path: path.to_string(),
                handler,
            }
        );
    }

    pub fn handle(&self, req: Request) -> Response {
        if let Some(logger) = &self.logger {
            logger(&req);
        }
        for route in &self.routes {
            if route.method == req.method && route.path == req.path {
                return (route.handler)(req);
            }
        }
        Response::not_found()
    }

    pub fn log(&mut self, logger: Logger) {
        self.logger = Some(logger);
    }
}