use crate::types::{Method, ParamMap, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Type alias of async handler function for usage in middleware.
/// A Handler has the following signature:
/// `async fn handler(Request) -> Response`
pub type Handler =
    Arc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;
type Middleware =
    Arc<dyn Fn(Request, Handler) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;
//type Logger = Arc<dyn Fn(&Request) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum RouteSegment {
    Static(Arc<str>),
    Param(Arc<str>),
}

#[derive(Clone)]
struct Route {
    method: Method,
    segments: Vec<RouteSegment>,
    handler: Handler,
    middleware: Option<Middleware>,
}

/// Router struct, contains routes and the methods needed to route requests to them.
#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
    //global_middleware: Option<Middleware>,
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
            //global_middleware: None,
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
        self.routes.push(Route {
            method,
            //path: Arc::from(path),
            segments: parse_route(path),
            handler,
            middleware: None,
        });
    }

    pub(crate) async fn handle_request(&self, mut req: Request) -> Response {
        /*if let Some(logger) = &self.logger {
            logger(&req).await;
        }*/
        for route in &self.routes {
            if route.method == req.method
                && let Some(params) = match_route(&route.segments, &req.path)
            {
                req.params = Arc::from(params);

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

fn match_route(route_segments: &Vec<RouteSegment>, req_path: &str) -> Option<ParamMap> {
    let req_segments: Vec<&str> = req_path.trim_matches('/').split('/').collect();

    if route_segments.len() != req_segments.len() {
        return None;
    }

    let mut params = ParamMap::new();

    for (route_segment, req_segment) in route_segments.iter().zip(req_segments.iter()) {
        match route_segment {
            RouteSegment::Static(seg) => {
                if seg.as_ref() != *req_segment {
                    return None;
                }
            }
            RouteSegment::Param(name) => {
                params.insert(name.clone(), Arc::from(*req_segment));
            }
        }
    }

    Some(params)
}

fn parse_route(path: &str) -> Vec<RouteSegment> {
    path.trim_matches('/')
        .split('/')
        .map(|s| {
            if let Some(stripped) = s.strip_prefix(':') {
                RouteSegment::Param(Arc::from(stripped))
            } else {
                RouteSegment::Static(Arc::from(s))
            }
        })
        .collect()
}