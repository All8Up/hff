mod api;
pub use api::*;

mod async_std_reader;
pub use async_std_reader::AsyncStdReader;

use async_std::io::{Read, Seek};
use core::marker::Unpin;

pub trait ReadSeek: Read + Seek + Unpin {}
impl<T: Read + Seek + Unpin> ReadSeek for T {}
