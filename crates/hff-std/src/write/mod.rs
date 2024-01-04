use crate::WriteSeek;
use hff_core::{
    write::{DataArray, DataSource, HffDesc},
    ByteOrder, Ecc, Header, Result,
};
use std::io::Write;

/// Writer trait for HffDesc.
pub trait Writer {
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

impl<'a> Writer for HffDesc<'a> {
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
        let _test = write_data_array(data, writer)?;
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
        let offset_len = write_data_array(data, &mut writer)?;

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

/// Write the data to the given stream.
/// Returns a vector of offset into the writer (starting from 0)
/// and the length of the data written without alignment padding.
fn write_data_array(data_array: DataArray, writer: &mut dyn Write) -> Result<Vec<(u64, u64)>> {
    let mut offset_len = vec![];

    // Track where we are in the writer, starting from zero.
    let mut offset = 0;
    for mut item in data_array {
        // Prepare each item.
        // This is only for compressed data (at this time) to perform
        // the compression.  Using std write here means it all has to
        // be buffered into memory.
        item.prepare()?;

        // Write in the appropriate manner.
        let length = match item {
            DataSource::File(mut f, _) => std::io::copy(&mut f, writer)?,
            DataSource::Owned(data) => std::io::copy(&mut data.as_slice(), writer)?,
            DataSource::Ref(mut data) => std::io::copy(&mut data, writer)?,
            #[cfg(feature = "compression")]
            DataSource::Compressed(_, _, data) => {
                std::io::copy(&mut data.unwrap().as_slice(), writer)?
            }
        };

        // Record the offset and length.
        offset_len.push((offset as u64, length));

        // What is the padding requirement?
        let padding = (length.next_multiple_of(16) - length) as usize;
        // Track where we are in the output stream.
        offset += length as usize + padding;

        // Write the padding.
        let padding = vec![0; padding];
        writer.write_all(&padding)?;
    }

    Ok(offset_len)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use hff_core::write::{chunk, hff, table};

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

        let hff = crate::read::inspect(&mut buffer.as_slice()).unwrap();
        println!("{:#?}", hff);
        println!("-----------------------------");
        for (depth, table) in hff.depth_first() {
            println!("-- <{}>: <{:?}>", depth, table);
        }
        println!("-----------------------------");

        //assert!(false);
    }
}
