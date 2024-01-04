mod api;
pub use api::*;

mod tokio_reader;
pub use tokio_reader::TokioReader;

use core::marker::Unpin;
use tokio::io::{AsyncRead, AsyncSeek};

pub trait ReadSeek: AsyncRead + AsyncSeek + Unpin {}
impl<T: AsyncRead + AsyncSeek + Unpin> ReadSeek for T {}
