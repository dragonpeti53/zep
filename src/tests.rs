#[cfg(test)]
mod tests {
    use crate::*;
    fn root(_req: Request) -> Response {
        Response::ok("true")
    }

    fn paramtest(req: Request) -> Response {
        Response::ok(
            if let Some(id) = req.params.get("id") {
                id
            } else {
                "error"
            }
        )
    }

    fn paramtest2(req: Request) -> Response {
        Response::ok(
            if let (Some(id1), Some(id2)) = (req.params.get("id1"), req.params.get("id2")) {
                id1.clone() + id2
            } else {
                "error".to_string()
            }
        )
    }

    #[tokio::test]
    async fn testrouter() {
        let mut router = Router::new();
        router.route(Method::GET, "/", root);
        
        
        let req = Request {
            method: Method::GET,
            path: "/".to_string(),
            version: Version::Other,
            headers: HeaderMap::new(),
            body:  Vec::new(),
            remote_addr: "".to_string(),
            params: ParamMap::new(),
        };

        let result = router.handle(req).await;

        let expected = Response {
            status_code: StatusCode::Ok,
            headers: HeaderMap::new(),
            body: "true".into()
        };

        assert_eq!(result, expected);
    }
    
    #[tokio::test]
    async fn testrouter2() {
        let mut router = Router::new();
        router.route(Method::GET, "/:id", paramtest);

        let req = Request {
            method: Method::GET,
            path: "/12".to_string(),
            version: Version::Other,
            headers: HeaderMap::new(),
            body:  Vec::new(),
            remote_addr: "".to_string(),
            params: ParamMap::new(),
        };

        let result = router.handle(req).await;

        let expected = Response {
            status_code: StatusCode::Ok,
            headers: HeaderMap::new(),
            body: "12".into()
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn testrouter3() {
        let mut router = Router::new();
        router.route(Method::GET, "/:id1/:id2", paramtest2);

        let req = Request {
            method: Method::GET,
            path: "/12/34".to_string(),
            version: Version::Other,
            headers: HeaderMap::new(),
            body:  Vec::new(),
            remote_addr: "".to_string(),
            params: ParamMap::new(),
        };

        let result = router.handle(req).await;

        let expected = Response {
            status_code: StatusCode::Ok,
            headers: HeaderMap::new(),
            body: "1234".into()
        };

        assert_eq!(result, expected);
    }
}