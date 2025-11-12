#[cfg(test)]
mod tests {
    use crate::*;
    fn root(_req: Request) -> Response {
        Response::ok("true")
    }

    #[tokio::test]
    async fn main() {
        let mut router = Router::new();
        router.route(Method::GET, "/", root);
        
        let req = Request {
            method: Method::GET,
            path: "/".to_string(),
            version: Version::Other,
            headers: Vec::new(),
            body:  Vec::new(),
            remote_addr: "".to_string(),
        };

        let result = router.handle(req);

        let expected = Response {
            status_code: StatusCode::Ok,
            body: "true".into()
        };

        assert_eq!(result, expected);
    }
    

}