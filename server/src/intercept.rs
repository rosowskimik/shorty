use tonic::{service::Interceptor, Status};

#[derive(Debug, Clone)]
pub struct TokenInterceptor {
    pub token: String,
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        if let Some(token) = req.metadata().get("x-auth-token") {
            let Ok(token) = token.to_str() else {
                return Err(Status::invalid_argument("Token is not valid UTF-8"));
            };

            if token == self.token {
                Ok(req)
            } else {
                Err(Status::unauthenticated("Invalid token provided"))
            }
        } else {
            Err(Status::unauthenticated("Token not provided"))
        }
    }
}
