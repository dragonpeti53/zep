type Handler = fn(Request) -> Response;
type Logger = fn(&Request);
type Middleware = fn(Request, Handler) -> Response;
type _SimpleHandler = fn(Request) -> Response;
type _ParamHandler = fn(Request, Vec<&str>) -> Response;

use crate::types::{Request, Response, Method};

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

    pub fn handle(&self, req: Request) -> Response {
        if let Some(logger) = &self.logger {
            logger(&req);
        }
        for route in &self.routes {
            if route.method == req.method && route.path == req.path {
                if let Some(middleware) = route.middleware {
                    return middleware(req, route.handler);
                } else {
                    return (route.handler)(req);
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