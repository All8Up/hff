use std::fs::File;

use async_std::stream::StreamExt;
use async_std::{
    fs::read_dir,
    path::{Path, PathBuf},
    task::spawn,
};
use hff_std::hff_core::{
    write::{ChunkDesc, TableDesc},
    Table,
};
use hff_std::utilities::{Ksv, StringVec};
use hff_std::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Structure {
    Directory(PathBuf, Vec<Structure>),
    File(PathBuf),
}

impl Structure {
    /// Create a new structure instance from the given path.
    pub async fn new(path: &Path, recursive: bool) -> Result<Self> {
        use normpath::PathExt;
        let path: PathBuf = std::path::PathBuf::from(path).normalize()?.as_path().into();

        if path.exists().await {
            let metadata = path.metadata().await?;
            let file_type = metadata.file_type();

            if file_type.is_file() {
                Ok(Self::File(path.into()))
            } else if file_type.is_dir() {
                Ok(Self::scan_directory(path.into(), recursive).await?)
            } else {
                Err(Error::Invalid(format!("Invalid root: {:?}", path)))
            }
        } else {
            Err(Error::Invalid(format!("Invalid root: {:?}", path)))
        }
    }

    /// Strip the given prefix from paths and as we descend,
    /// add the parents to what is stripped.  NOTE: Requires that
    /// the parent is stripped of everything except the final component
    /// in order to function correctly.
    pub fn strip_prefix(self, prefix: &Path) -> Result<Self> {
        match self {
            Self::Directory(path, children) => {
                // Remove the prefix from the path portion.
                let p = path.strip_prefix(prefix)?;
                // As we descend, add the remaining prefix to the prefix
                // sent to the children.
                let cp = prefix.join(p);

                let mut c = vec![];
                for child in children {
                    c.push(child.strip_prefix(&cp)?);
                }
                Ok(Self::Directory(p.into(), c))
            }
            Self::File(path) => Ok(Self::File(path)),
        }
    }

    /// Scan the given directory for files and child directories.
    #[async_recursion::async_recursion]
    async fn scan_directory(path: PathBuf, recursive: bool) -> Result<Self> {
        let mut result = vec![];
        let mut directories = vec![];
        let mut reader = read_dir(&path).await?;

        while let Some(entry) = reader.next().await {
            match entry {
                Ok(entry) => {
                    let metadata = entry.metadata().await?;
                    if metadata.file_type().is_file() {
                        result.push(Self::File(entry.path().file_name().unwrap().into()));
                    } else if metadata.file_type().is_dir() {
                        if recursive {
                            let path = entry.path();
                            directories.push(spawn(Self::scan_directory(path, recursive)));
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        while let Some(child) = directories.pop() {
            result.push(child.await?);
        }

        Ok(Self::Directory(path.into(), result))
    }

    /// Convert the structure into a set of tables.
    pub fn to_tables<'a, E: ByteOrder>(
        self,
        root: &Path,
        compression: impl Fn(&Path) -> Option<u32>,
    ) -> Result<Vec<TableBuilder<'a>>> {
        match self {
            Self::File(file) => archive_single_file::<E>(root, file, &compression),
            Self::Directory(path, children) => archive_directory(path, children, &compression),
        }
    }
}

fn archive_directory<'a>(
    path: PathBuf,
    children: Vec<Structure>,
    compression: &impl Fn(&Path) -> Option<u32>,
) -> Result<Vec<TableBuilder<'a>>> {
    // The metadata attached to this table will describe the names
    // of the directory and the files which are stored in the chunks.
    let mut ksv = Ksv::new();

    // Insert the path of the root directory for everything we find.
    // This is just the name and not the full path.
    ksv.insert(
        "dir".into(),
        [path.display().to_string()].into_iter().into(),
    );

    println!("{:?}", ksv);

    // Build children tables and chunks.
    let mut tables = vec![];
    let mut chunks = vec![];

    for child in children {
        match child {
            Structure::File(file) => chunks.push(chunk(
                super::HFF_FILE,
                Ecc::INVALID,
                file.display().to_string(),
            )?),
            Structure::Directory(..) => {
                println!("D: {:?}", child);
                tables.push(child);
            }
        }
    }

    Ok(vec![])
}

/// Given a single file, turn it into a table entry.
/// This is used to pack a single file into an archive.
fn archive_single_file<'a, E: ByteOrder>(
    root: &Path,
    file: PathBuf,
    compression: &impl Fn(&Path) -> Option<u32>,
) -> Result<Vec<TableBuilder<'a>>> {
    // Build the path to the file.
    let file_path = root.join(&file);

    // Create the metadata for the file.
    let mut ksv = Ksv::new();
    ksv.insert(
        "file".into(),
        StringVec::from([file.display().to_string()].into_iter()),
    );

    // Attempt to open the file as an hff first.
    match hff_std::open(File::open(&file_path)?) {
        Ok(hff) => Ok(vec![hff_to_table::<E>(root, file, hff, compression)?]),
        Err(_) => {
            // The file is not an hff, so just pack it into a chunk.
            let file_path: std::path::PathBuf = file_path.into();
            let chunk = file_to_chunk(compression, file_path)?;
            Ok(vec![table(super::HFF_FILE, Ecc::INVALID)
                .chunks([chunk])
                .metadata(ksv.to_bytes::<E>()?)?])
        }
    }
}

/// Given an hff file, convert it into a decomposed table.
fn hff_to_table<'a, E: ByteOrder>(
    root: &Path,
    file: PathBuf,
    hff: Hff<StdReader>,
    compression: &impl Fn(&Path) -> Option<u32>,
) -> Result<TableBuilder<'a>> {
    // Create the metadata for the file.
    let mut ksv = Ksv::new();
    ksv.insert(
        "file".into(),
        StringVec::from([file.display().to_string()].into_iter()),
    );

    // Convert content for embedding.
    let mut children = vec![];
    for t in hff.tables() {
        children.push(resolve_table(&hff, t)?);
    }

    // This new table contains the content of the prior hff.
    // At the root level of an hff, it can only contain tables, so
    // this new table has no chunks, only the original children tables.
    let result = table(super::HFF_EMBEDDED, hff.content_type())
        .metadata(ksv.to_bytes::<E>()?)?
        .children(children.into_iter());

    Ok(result)
}

fn resolve_table<'a, 'b>(
    hff: &'a Hff<StdReader>,
    t: TableView<'a, StdReader>,
) -> Result<TableBuilder<'b>> {
    // Get all the chunks.
    let mut chunks = vec![];
    for c in t.chunks() {
        chunks.push(chunk(c.primary(), c.secondary(), hff.get(&c)?)?);
    }

    // Recursively resolve all child tables.
    let mut children = vec![];
    for t in t.iter() {
        children.push(resolve_table(hff, t)?);
    }

    // And build the whole thing as a table in the archive.
    Ok(table(t.primary(), t.secondary())
        .chunks(chunks.into_iter())
        .children(children.into_iter()))
}

/// Convert the given file into a chunk without decomposition.
fn file_to_chunk<'a, F: Fn(&Path) -> Option<u32>>(
    compression: &F,
    file_path: std::path::PathBuf,
) -> Result<ChunkDesc<'a>> {
    let compression = compression(file_path.as_path().into());
    let chunk = if let Some(compression) = compression {
        chunk(super::HFF_FILE, super::HFF_LZMA, (compression, file_path))?
    } else {
        chunk(super::HFF_FILE, Ecc::INVALID, file_path)?
    };
    Ok(chunk)
}
