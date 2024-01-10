use crate::{ContentInfo, Result};
use std::io::Read;

/// Decompress the provided data.
#[cfg(feature = "compression")]
pub fn decompress(source: &[u8]) -> Result<Vec<u8>> {
    if source.len() > 0 {
        let mut decoder = xz2::read::XzDecoder::new(source);
        let mut result = vec![];
        std::io::copy(&mut decoder, &mut result)?;
        Ok(result)
    } else {
        Ok(vec![])
    }
}
