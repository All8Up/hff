use async_std::fs::read_dir;
use byteorder::ByteOrder;
use hff_std::{hff_core::utilities::Ksv, *};
use normpath::PathExt;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

/// An archive entry.
pub const HFF_ARCHIVE: Ecc = Ecc::new("_ARCHIVE");
/// This is the type for a directory table entry.
pub const HFF_DIR: Ecc = Ecc::new("_DIR");
/// This is the type for a file chunk.
pub const HFF_FILE: Ecc = Ecc::new("_FILE");
/// If the chunk is compressed.
pub const HFF_LZMA: Ecc = Ecc::new("_LZMA");

/// Represents a directory tree for inserting into a Hff file.
#[derive(Debug)]
pub struct Hierarchy {
    path: PathBuf,
    files: Vec<PathBuf>,
    children: Vec<Hierarchy>,
}

impl Hierarchy {
    /// Create a new directory instance and populate it.
    pub async fn new(root: &Path, recurse: bool) -> Result<Self> {
        // Make sure the path is normalized (not canonicalized).
        let normalized = root.normalize()?;
        let source: &Path = normalized.as_path();

        // Use the parent of the given path as the stored name.
        let parent = source.parent().unwrap_or(source);

        // Build the structure from here.
        let directories = Self::build(parent, source, recurse).await?;

        Ok(directories)
    }

    /// Recursively build directory instances.
    /// The root path will be stripped off the source path prior to
    /// storage.  This allows the structure to be internally relative.
    #[async_recursion::async_recursion]
    async fn build(root: &Path, source: &Path, recurse: bool) -> Result<Self> {
        // Kick off the recursion.
        let (directories, files) = Self::scan_content(source).await?;

        // Process each child.
        let mut children = vec![];
        if recurse {
            for directory in directories {
                children.push(Self::build(source, &directory, recurse).await?);
            }
        }

        Ok(Self {
            path: source.strip_prefix(root)?.into(),
            files,
            children,
        })
    }

    /// Get a string table representing the content of this directory.
    pub fn as_ksv(&self) -> Result<Ksv> {
        let mut ksv = Ksv::new();

        // Write in the directory name as the first string.
        ksv.insert(
            "dir".into(),
            [self.path.display().to_string()].into_iter().into(),
        );

        // Write each of the file entries.
        let mut files = hff_std::hff_core::utilities::StringVec::new();
        for entry in &self.files {
            files.push(entry.display().to_string());
        }
        ksv.insert("files".into(), files);

        Ok(ksv)
    }

    /// Create a directory structure from a string table.
    #[allow(unused)]
    pub fn from_ksv(ksv: Ksv) -> Result<Self> {
        Ok(Self {
            path: (&ksv["dir"][0]).into(),
            files: ksv["files"].into_iter().map(|s| s.into()).collect(),
            children: vec![],
        })
    }

    /// Write the entire thing to a builder.
    pub fn write(
        self,
        source: &Path,
        compression_level: Option<u32>,
        metadata: Option<Vec<u8>>,
    ) -> Result<hff::TableBuilder> {
        // Get the parent of source in order to strip it from the output.
        let source = if let Some(source) = source.parent() {
            source
        } else {
            source
        };

        // Create the root table.
        // let mut hff = hff(HFF_ARCHIVE)?;

        // hff = if let Some(metadata) = metadata {
        //     hff.metadata(metadata)?
        // } else {
        //     hff
        // };

        // Write the hierarchy.
        Ok(self.write_tables(source, hff, compression_level)?)
    }

    /// Write the entire structure to an hff builder.
    fn write_tables(
        self,
        source: &Path,
        mut hff: hff::TableBuilder,
        compression_level: Option<u32>,
    ) -> Result<hff::TableBuilder> {
        // Create a full path to the source item.
        // let source_dir = source.join(&self.path);

        // // Add a table with the appropriate metadata for this entry.
        // let metadata = self.as_ksv()?;

        // // Add the current level as a table with metadata describing the content.
        // let mut table = hff::table(HFF_DIR)?.metadata(metadata)?;

        // // And add child tables for the contained directories.
        // for child in self.children {
        //     // Recurse.
        //     table = child.write_tables(&source_dir, table, compression_level)?;
        // }

        // hff = hff.child(table)?;

        // // Add the files as chunks.
        // // The key item to keep in mind here is that the order of chunks
        // // is stable within a table.  So, adding them here in the same
        // // order that they are added to the metadata will maintain the
        // // 1:1 mapping.
        // for file in self.files {
        //     // Just push a PathBuf into the hff for each chunk.
        //     if let Some(level) = &compression_level {
        //         hff = hff.chunk(
        //             HFF_FILE,
        //             HFF_LZMA,
        //             Compress::with(source_dir.join(file), *level),
        //         )?;
        //     } else {
        //         hff = hff.chunk(HFF_FILE, Ecc::INVALID, source_dir.join(file))?;
        //     }
        // }

        Ok(hff)
    }

    /// Read the archive from the given reader.
    #[allow(unused)]
    pub async fn read<E: ByteOrder>(path: &Path) -> Result<Self> {
        // Uses the random access variation for proofing reasons.
        // This only recovers the structure.
        let hff = open(File::open(path)?)?;

        // Check that this is an archive.
        if hff.content_type() == HFF_ARCHIVE {
            // Read the tables.
            let mut children = vec![];
            for (_, table) in hff.depth_first() {
                children.push(Self::read_child::<E>(&hff, table).await?);
            }

            Ok(Self {
                // The root is just a container, so no path.
                path: "".into(),
                // The root has no files, at least the way this is setup right now.
                files: vec![],
                // And it has the children we just read.
                children: children,
            })
        } else {
            Err(Error::Invalid("Not an Archive file.".into()))
        }
    }

    /// Read a child directory (and potential children).
    #[async_recursion::async_recursion]
    async fn read_child<E: ByteOrder>(hff: &hff::AsyncHff, table: &hff::Table) -> Result<Self> {
        // Get the metadata.
        let metadata = hff.metadata(table).await?;
        let string_table = StringTable::read::<E>(&mut metadata.as_slice())?;
        let mut directory = Self::from_ksv(string_table)?;

        // Read the children.
        for child in table.children() {
            directory
                .children
                .push(Self::read_child::<E>(hff, child).await?);
        }

        Ok(directory)
    }

    /// Scan the content of the given directory, returns child directory list and
    /// the files within the directory.
    async fn scan_content(directory: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        use async_std::stream::StreamExt;

        let mut directories: Vec<PathBuf> = vec![];
        let mut files: Vec<PathBuf> = vec![];

        let mut reader = read_dir(directory).await?;
        while let Some(entry) = reader.next().await {
            match entry {
                Ok(entry) => {
                    let metadata = entry.metadata().await?;
                    if metadata.file_type().is_file() {
                        files.push(PathBuf::from(entry.path().file_name().unwrap()));
                    }
                    if metadata.file_type().is_dir() {
                        directories.push(entry.path().into())
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok((directories, files))
    }
}
