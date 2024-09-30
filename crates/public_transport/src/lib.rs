use std::error::Error;

use model::{agency::Agency, WithId, WithOrigin};
use tokio::sync::{mpsc, oneshot};

pub mod client;
pub mod collector;
pub mod database;
pub mod server;

#[derive(Debug)]
pub enum RequestError {
    NotFound,
    IdMissing,
    SendError(mpsc::error::SendError<Request>),
    ResponseError(oneshot::error::RecvError),
    Other(Box<dyn Error + Send>),
}

impl RequestError {
    pub fn other<T: Error + Send + 'static>(why: T) -> Self {
        Self::Other(Box::new(why))
    }
}

impl From<Box<dyn Error + Send>> for RequestError {
    fn from(value: Box<dyn Error + Send>) -> Self {
        RequestError::Other(value)
    }
}

impl From<database::DatabaseError> for RequestError {
    fn from(value: database::DatabaseError) -> Self {
        match value {
            database::DatabaseError::NotFound => Self::NotFound,
            database::DatabaseError::IdMissing => Self::IdMissing,
            database::DatabaseError::Other(why) => Self::Other(why),
        }
    }
}

impl From<mpsc::error::SendError<Request>> for RequestError {
    fn from(why: mpsc::error::SendError<Request>) -> Self {
        Self::SendError(why)
    }
}

impl From<oneshot::error::RecvError> for RequestError {
    fn from(why: oneshot::error::RecvError) -> Self {
        Self::ResponseError(why)
    }
}

pub type RequestResult<O> = Result<O, RequestError>;

pub fn not_found_to_none<O>(result: RequestResult<O>) -> RequestResult<Option<O>> {
    if let Err(RequestError::NotFound) = result {
        Ok(None)
    } else {
        result.map(Some)
    }
}

#[derive(Debug)]
pub enum Request {
    PushAgency {
        agency: WithOrigin<Agency>,
        responder: oneshot::Sender<RequestResult<WithOrigin<WithId<Agency>>>>,
    },
}
