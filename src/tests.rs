#[cfg(test)]
mod tests {
    use crate::*;
    async fn root(_req: Request) -> Response {
        Response::ok("true")
    }

    async fn paramtest(req: Request) -> Response {
        Response::ok(if let Some(id) = req.params.get("id") {
            id.to_string()
        } else {
            "error".to_string()
        })
    }

    async fn paramtest2(req: Request) -> Response {
        Response::ok(
            if let (Some(id1), Some(id2)) = (req.params.get("id1"), req.params.get("id2")) {
                id1.to_string() + id2
            } else {
                "error".to_string()
            },
        )
    }

    #[tokio::test]
    async fn testrouter() {
        let mut router = Router::new();
        router.route(Method::GET, "/", root);

        let req = Request {
            method: Method::GET,
            path: "/".to_string().into(),
            ..Default::default()
        };

        let result = router.handle_request(req).await;

        let expected = Response {
            status_code: StatusCode::Ok,
            headers: None,
            body: Some("true".into()),
            stream: None,
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn testrouter2() {
        let mut router = Router::new();
        router.route(Method::GET, "/:id", paramtest);

        let req = Request {
            method: Method::GET,
            path: "/12".to_string().into(),
            ..Default::default()
        };

        let result = router.handle_request(req).await;

        let expected = Response {
            status_code: StatusCode::Ok,
            headers: None,
            body: Some("12".into()),
            stream: None,
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn testrouter3() {
        let mut router = Router::new();
        router.route(Method::GET, "/:id1/:id2", paramtest2);

        let req = Request {
            method: Method::GET,
            path: "/12/34".to_string().into(),
            ..Default::default()
        };

        let result = router.handle_request(req).await;

        let expected = Response {
            status_code: StatusCode::Ok,
            headers: None,
            body: Some("1234".into()),
            stream: None,
        };

        assert_eq!(result, expected);
    }
}
