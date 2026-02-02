use grpc_api::api::*;
use crate::errors::CustomError;


use tonic::{Request, Response, Status};

pub struct UsersService {
    pub pool: Pool, 
}