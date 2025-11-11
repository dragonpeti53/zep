type Handler = fn(Request) -> Response;

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
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
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
        for route in &self.routes {
            if route.method == req.method && route.path == req.path {
                return (route.handler)(req);
            }
        }
        Response::not_found()
    }
}