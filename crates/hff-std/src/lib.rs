//! Implements the basic reader/writer functionality for HFF.
#![warn(missing_docs)]
use hff_core::{Ecc, Header, Result};

mod chunk_desc;
pub use chunk_desc::ChunkDesc;

mod table_desc;
pub use table_desc::TableDesc;

mod table_builder;
pub use table_builder::TableBuilder;

/// Create a table structure to be contained in the HFF.
/// Panics if the given Ecc data is invalid.
pub fn table(primary: impl Into<Ecc>, secondary: impl Into<Ecc>) -> TableBuilder {
    TableBuilder::new(primary.into(), secondary.into())
}

/// Write the table structure to an HFF container.
use hff_core::ByteOrder;

pub fn write_to<E: ByteOrder>(
    content_type: impl Into<Ecc>,
    content: TableDesc,
    writer: &mut dyn std::io::Write,
) -> Result<()> {
    use hff_core::Header;
    let header = Header::new(content_type.into(), 0, 0);

    header.write::<E>(writer)?;

    Ok(())
}

pub fn read_from(reader: &mut dyn std::io::Read) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        // Simple dev test for structure.
        let table = table("Prime", "Sub").end();
        println!("{:#?}", table);

        let mut buffer = vec![];

        assert!(write_to::<hff_core::NE>("Dumb", table, &mut buffer).is_ok());
        println!("{:#?}", buffer);
        assert!(read_from(&mut buffer.as_slice()).is_ok());

        assert!(false);
    }
}
