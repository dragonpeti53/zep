use std::future::Future;
use std::pin::Pin;

use std::sync::Arc;

pub type Handler = Arc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;
pub type Middleware = Arc<dyn Fn(Request, Handler) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;


//pub type Logger = Arc<dyn Fn(&Request) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;





//pub type Handler = fn(Request) -> Response;
//type Logger = fn(&Request);
//type Middleware = fn(Request, Handler) -> Response;


use crate::types::{Request, Response, Method, ParamMap};



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
    //logger: Option<Logger>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            //logger: None,
        }
    }

    pub fn route<F, Fut>(&mut self, method: Method, path: &str, f: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: Handler = Arc::new(move |req| Box::pin(f(req)));
        self.routes.push(
            Route {
                method,
                path: path.to_string(),
                handler,
                middleware: None,
            }
        );
    }

    pub async fn handle(&self, mut req: Request) -> Response {
        /*if let Some(logger) = &self.logger {
            logger(&req).await;
        }*/
        for route in &self.routes {
            if route.method == req.method
                && let Some(params) = match_route(&route.path, &req.path) {
                    req.params = params;

                    if let Some(middleware) = route.middleware.clone() {
                        return middleware(req, route.handler.clone()).await;
                    } else {
                        return (route.handler)(req).await;
                    }
                }
        }
        Response::not_found()
    }

    /*
    pub fn log<F, Fut>(&mut self, f: F)
    where
        F: Fn(&Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let logger: Logger = Arc::new(move |req| Box::pin(f(req)));
        self.logger = Some(logger);
    }*/



    pub fn middleware<F, Fut>(&mut self, f: F)
    where
        F: Fn(Request, Handler) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        if let Some(route) = self.routes.last_mut() {
            route.middleware = Some(Arc::new(move |req, next| Box::pin(f(req, next))));
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
