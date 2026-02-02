use shared::cvad_restapi;

// Generated from build.rs (prost/tonic output)
pub mod generated {
    pub mod cvad {
        pub mod v1 {
            include!("generated/cvad.v1.rs");
        }
    }
}

use generated::cvad::v1::{GetExpireDateRequest, GetExpireDateResponse, cvad_server::Cvad};

use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct CvadService;

#[tonic::async_trait]
impl Cvad for CvadService {
    async fn get_expire_date(
        &self,
        request: Request<GetExpireDateRequest>,
    ) -> Result<Response<GetExpireDateResponse>, Status> {
        let req = request.into_inner();

        // Basic input validation
        if req.cvad_url.trim().is_empty() || req.site_name.trim().is_empty() {
            return Ok(Response::new(GetExpireDateResponse {
                expire_rfc3339: String::new(),
                found: false,
                error: "cvad_url and site_name are required".to_string(),
            }));
        }

        // Call your existing REST API wrapper
        let api =
            cvad_restapi::CvadRestApi::new(req.cvad_url.as_str(), req.site_name.as_str()).await;
        let expire_opt = api.get_expire_date().await;

        let (expire_rfc3339, found) = match expire_opt {
            Some(s) => (s, true),
            None => (String::new(), false),
        };

        Ok(Response::new(GetExpireDateResponse {
            expire_rfc3339,
            found,
            error: String::new(),
        }))
    }
}
