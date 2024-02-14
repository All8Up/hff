use crate::Result;

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

/// Decompress the input to the output destination, decompressed data
/// must match the exact length of the destination.
#[cfg(feature = "compression")]
pub fn decompress_exact(source: &[u8], mut destination: &mut [u8]) -> Result<()> {
    use crate::Error;

    if source.len() > 0 && destination.len() > source.len() {
        let mut decoder = xz2::read::XzDecoder::new(source);
        std::io::copy(&mut decoder, &mut destination)?;
        Ok(())
    } else {
        Err(Error::Invalid(
            "Incorrect inputs to decompress_exact.".into(),
        ))
    }
}
