use hff_core::{write::HffDesc, ByteOrder, Ecc, Header, Result};
use std::io::{Seek, Write};

/// Helper trait for lazy writing.
pub trait WriteSeek: Write + Seek {}

/// Blanket implementation for anything viable.
impl<T: Write + Seek> WriteSeek for T {}

/// Writer trait for HffDesc.
pub trait StdWriter {
    /// Write to a stream.
    fn write<E: ByteOrder>(
        self,
        content_type: impl Into<Ecc>,
        writer: &mut dyn Write,
    ) -> Result<()>;

    /// Write to a stream but finalize chunk lazilly during the write.
    /// This requires a stream with both Write and Seek capabilities.
    fn lazy_write<E: ByteOrder>(
        self,
        content_type: impl Into<Ecc>,
        writer: &mut dyn WriteSeek,
    ) -> Result<()>;
}

impl<'a> StdWriter for HffDesc<'a> {
    fn write<E: ByteOrder>(
        self,
        content_type: impl Into<Ecc>,
        writer: &mut dyn Write,
    ) -> Result<()> {
        let offset_to_blob = self.offset_to_blob() as u64;
        let (mut tables, mut chunks, mut data) = self.finish();

        let header = Header::new(
            content_type.into(),
            tables.len() as u32,
            chunks.len() as u32,
        );
        writer.write_all(header.to_bytes::<E>()?.as_slice())?;

        // Prepare all the data in the data array so we have offsets and length.
        let offset_len = data.prepare()?;

        // Update the table metadata length/offset and chunk length/offset.
        HffDesc::update_data(&mut tables, &mut chunks, offset_to_blob, &offset_len);

        // And write the content+data blob.
        writer.write_all(tables.to_bytes::<E>()?.as_slice())?;
        writer.write_all(chunks.to_bytes::<E>()?.as_slice())?;
        let _test = data.write(writer)?;
        assert_eq!(_test, offset_len);

        Ok(())
    }

    fn lazy_write<E: ByteOrder>(
        self,
        content_type: impl Into<Ecc>,
        mut writer: &mut dyn WriteSeek,
    ) -> Result<()> {
        let array_size = self.arrays_size();
        let offset_to_blob = self.offset_to_blob() as u64;
        let (mut tables, mut chunks, data) = self.finish();

        let header = Header::new(
            content_type.into(),
            tables.len() as u32,
            chunks.len() as u32,
        );
        writer.write_all(header.to_bytes::<E>()?.as_slice())?;

        // Write zero's for the table and chunk array.
        // Use this rather than skipping in order to avoid any questionable
        // differences between different backing types.
        writer.write_all(&mut vec![0; array_size])?;

        // Write the data and record the offset/length information.
        let offset_len = data.write(&mut writer)?;

        // Update the table metadata length/offset and chunk length/offset.
        HffDesc::update_data(&mut tables, &mut chunks, offset_to_blob, &offset_len);

        // Seek back to the tables/chunks.
        writer.seek(std::io::SeekFrom::Start(Header::SIZE as u64))?;

        // And write the tables and chunks.
        writer.write_all(tables.to_bytes::<E>()?.as_slice())?;
        writer.write_all(chunks.to_bytes::<E>()?.as_slice())?;

        Ok(())
    }
}

/*

/// Write the content to the given stream.  This requires seek because we
/// update the table and chunk entries 'after' writing the data blob and as
/// such, we have to go back and write them.
pub fn lazy_write<E: ByteOrder>(
    mut self,
    content_type: impl Into<Ecc>,
    mut writer: &mut dyn WriteSeek,
) -> Result<()> {
    self.write_header::<E>(content_type.into(), &mut writer)?;

    // Write zero's for the table and chunk array.
    // Use this rather than skipping in order to avoid any questionable
    // differences between different backing types.
    writer.write_all(&mut vec![0; self.arrays_size()])?;

    // Write the data and record the offset/length information.
    let data = self.data.take().unwrap();
    let offset_len = data.write(&mut writer)?;

    // Update the table metadata length/offset and chunk length/offset.
    self.update_data(self.offset_to_blob() as u64, &offset_len);

    // Seek back to the tables/chunks.
    writer.seek(std::io::SeekFrom::Start(Header::SIZE as u64))?;

    // And write the tables and chunks.
    writer.write_all(self.tables.to_bytes::<E>()?.as_slice())?;
    writer.write_all(self.chunks.to_bytes::<E>()?.as_slice())?;

    Ok(())
} */

#[cfg(test)]
mod tests {
    use crate::*;
    use hff_core::{
        read::Hff,
        write::{chunk, hff, table},
    };

    #[test]
    fn test() {
        let content = hff([
            table("p0", "s0")
                .metadata("123")
                .unwrap()
                .children([table("p1", "s1")
                    .metadata("1234")
                    .unwrap()
                    .chunks([
                        chunk("c0", "cs0", "chunk 0").unwrap(),
                        chunk("c1", "cs1", "chunk 1").unwrap(),
                        chunk("c2", "cs2", "chunk 2").unwrap(),
                    ])
                    .children([
                        table("p2", "s2").metadata("12345").unwrap().chunks([]),
                        table("p3", "s3").metadata("123456").unwrap().chunks([]),
                    ])])
                .chunks([]),
            table("p4", "s4").metadata("1234567").unwrap(),
            table("p5", "s5")
                .metadata("12345678")
                .unwrap()
                .chunks([chunk("c3", "cs3", "chunk 3").unwrap()]),
        ]);

        let mut buffer = vec![];
        content.write::<hff_core::LE>("Test", &mut buffer).unwrap();

        let (hff, _cache) = Hff::read_full(&mut buffer.as_slice()).unwrap();
        println!("{:#?}", hff);
        println!("-----------------------------");
        for (depth, table) in hff.depth_first() {
            println!("-- <{}>: <{:?}>", depth, table);
        }
        println!("-----------------------------");

        //assert!(false);
    }
}
