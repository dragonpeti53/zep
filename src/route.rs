type Handler = fn(Request) -> Response;
type Logger = fn(&Request);
type Middleware = fn(Request, Handler) -> Response;
type _SimpleHandler = fn(Request) -> Response;
type _ParamHandler = fn(Request, Vec<&str>) -> Response;

use crate::types::{Request, Response, Method, ParamMap};

/*#[derive(Clone)]
enum Handler {
    SimpleHandler(SimpleHandler),
    ParamHandler(ParamHandler),
}*/

#[derive(Clone)]
struct Route {
    method: Method,
    path: String,
    handler: Handler,
    middleware: Option<Middleware>,
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

    pub fn route(&mut self, method: Method, path: &str, handler: Handler) {
        self.routes.push(
            Route {
                method: method,
                path: path.to_string(),
                handler,
                middleware: None,
            }
        );
    }

    pub async fn handle(&self, mut req: Request) -> Response {
        if let Some(logger) = &self.logger {
            logger(&req);
        }
        for route in &self.routes {
            if route.method == req.method {
                if let Some(params) = match_route(&route.path, &req.path) {
                    req.params = params;

                    if let Some(middleware) = route.middleware {
                        return middleware(req, route.handler);
                    } else {
                        return (route.handler)(req);
                    }
                }
            }
        }
        Response::not_found()
    }

    pub fn log(&mut self, logger: Logger) {
        self.logger = Some(logger);
    }

    pub fn middleware(&mut self, middleware: Middleware) {
        if let Some(route) = self.routes.last_mut() {
            route.middleware = Some(middleware);
        }
    }
}

fn match_route(route_path: &str, req_path: &str) -> Option<ParamMap> {
    let route_segments: Vec<&str> = route_path.trim_matches('/').split('/').collect();
    let req_segments: Vec<&str> = req_path.trim_matches('/').split('/').collect();

    if route_segments.len() != req_segments.len() {
        return None;
    }

    let mut params = ParamMap::new();

    for (r_seg, req_seg) in route_segments.iter().zip(req_segments.iter()) {
        if r_seg.starts_with(':') {
            params.insert(r_seg[1..].to_string(), req_seg.to_string());
        } else if r_seg != req_seg {
            return None;
        }
    }

    Some(params)
}
