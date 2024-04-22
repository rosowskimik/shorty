use tonic::{service::Interceptor, Status};

#[derive(Debug, Clone)]
pub struct TokenInterceptor {
    pub token: Option<String>,
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        if let Some(ref token) = self.token {
            req.metadata_mut()
                .insert("x-auth-token", token.parse().unwrap());
        }

        Ok(req)
    }
}
