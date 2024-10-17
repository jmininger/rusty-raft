use std::{
    error::Error,
    future::{
        pending,
        Future,
    },
};

use serde::{
    Deserialize,
    Serialize,
};
use tokio_util::either::Either;

// Can only increase if an election starts
pub type TermId = u64;

pub type LogIndex = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: TermId,
    pub message: Vec<u8>,
    pub index: LogIndex,
}

/// Util function that lets us use Option<impl Future<_>> in a select! block
pub fn dynamic_fut<T, F, E>(maybe_rx: Option<F>) -> impl Future<Output = Result<T, E>>
where
    F: Future<Output = Result<T, E>>,
    E: Error,
{
    match maybe_rx {
        Some(rx) => Either::Left(rx),
        None => Either::Right(pending()),
    }
}
