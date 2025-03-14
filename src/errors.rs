use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Server error: {0}")]
    ServerError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = match &self {
            AppError::InvalidAddress(_) => Status::BadRequest,
            AppError::NotFound(_) => Status::NotFound,
            AppError::RpcError(_) => Status::ServiceUnavailable,
            AppError::ServerError(_) => Status::InternalServerError,
        };

        let error_response = ErrorResponse {
            error: self.to_string(),
        };

        Response::build_from(Json(error_response).respond_to(req)?)
            .status(status)
            .ok()
    }
}
