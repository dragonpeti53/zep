use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::types::{Request, Response, Method, ParamMap};

/// Type alias of async handler function for usage in middleware.
/// A Handler has the following signature:
/// `async fn handler(Request) -> Response`
pub type Handler = Arc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;
type Middleware = Arc<dyn Fn(Request, Handler) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;






#[derive(Clone)]
struct Route {
    method: Method,
    path: String,
    handler: Handler,
    middleware: Option<Middleware>,
}

/// Router struct, contains routes and the methods needed to route requests to them.
#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    /// Returns a new Router struct.
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            //logger: None,
        }
    }

    /// Appends a new route to a router struct.
    /// Requires a method, path and handler function.
    /// # Example:
    /// ```
    /// use zep::{Router, Method, Request, Response};
    /// 
    /// async fn handler(_req: Request) -> Response {
    ///     Response::ok("Hello World!")
    /// }
    /// 
    /// let mut router = Router::new();
    /// router.route(Method::GET, "/", handler);
    /// ```
    pub fn route<F, Fut>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let handler: Handler = Arc::new(move |req| Box::pin(handler(req)));
        self.routes.push(
            Route {
                method,
                path: path.to_string(),
                handler,
                middleware: None,
            }
        );
    }

    pub(crate) async fn handle(&self, mut req: Request) -> Response {
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


    /// Appends a middleware to the latest route.
    /// Requires a function with the following signature:
    /// `async fn middleware(Request, Handler) -> Response`
    /// 
    /// # Example:
    /// 
    /// ```
    /// use zep::{Router, Method, Request, Response, Handler};
    /// 
    /// async fn handler(_req: Request) -> Response {
    ///     Response::ok("Hello World!")
    /// }
    /// 
    /// async fn middleware(req: Request, handler: Handler) -> Response {
    ///     //do stuff
    ///     return handler(req).await;
    /// }
    /// 
    /// let mut router = Router::new();
    /// router.route(Method::GET, "/", handler);
    /// router.middleware(middleware);
    /// //middleware is now applied to the `GET /` route.
    /// ```
    /// 
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
