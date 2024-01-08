use hff_std::hff_core::write::ChunkDesc;
use hff_std::utilities::{Ksv, StringVec};
use hff_std::*;
use std::{
    fs::{read_dir, File},
    path::{Path, PathBuf},
};

/// Structure of a scanned directory.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Structure {
    /// A directory entry.
    Directory(PathBuf, Vec<Structure>),
    /// A file entry.
    File(PathBuf),
}

impl Structure {
    /// Create a new structure instance from the given path.
    pub fn new(path: &Path, recursive: bool) -> Result<Self> {
        use normpath::PathExt;
        let path: PathBuf = path.normalize()?.into();

        if path.exists() {
            let metadata = path.metadata()?;
            let file_type = metadata.file_type();

            if file_type.is_file() {
                Ok(Self::File(path.into()))
            } else if file_type.is_dir() {
                Ok(Self::scan_directory(path.into(), recursive)?)
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
    fn scan_directory(path: PathBuf, recursive: bool) -> Result<Self> {
        let mut result = vec![];
        let mut reader = read_dir(&path)?;

        while let Some(entry) = reader.next() {
            match entry {
                Ok(entry) => {
                    let metadata = entry.metadata()?;
                    if metadata.file_type().is_file() {
                        result.push(Self::File(entry.path().file_name().unwrap().into()));
                    } else if metadata.file_type().is_dir() {
                        if recursive {
                            let path = entry.path();
                            result.push(Self::scan_directory(path, recursive)?);
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Self::Directory(path.into(), result))
    }

    /// Convert the structure into a set of tables.
    pub fn to_tables<'a, E: ByteOrder>(
        self,
        root: &Path,
        compression: impl Fn(&Path) -> Option<u32>,
    ) -> Result<TableBuilder<'a>> {
        match self {
            Self::File(file) => archive_single_file::<E>(root, file, &compression),
            Self::Directory(path, children) => {
                archive_directory::<E>(root, path, children, &compression)
            }
        }
    }
}

fn archive_directory<'a, E: ByteOrder>(
    root: &Path,
    path: PathBuf,
    structure: Vec<Structure>,
    compression: &impl Fn(&Path) -> Option<u32>,
) -> Result<TableBuilder<'a>> {
    // The metadata attached to this table will describe the names
    // of the directory and the files which are stored in the chunks.
    let mut ksv = Ksv::new();

    // Insert the path of the root directory for everything we find.
    // This is just the name and not the full path.
    ksv.insert(
        "dir".into(),
        [path.display().to_string()].into_iter().into(),
    );

    // Build children tables and chunks.
    let (tables, chunks, files) = archive_level::<E>(root, path, structure, compression)?;
    // Insert the file names into the metadata.
    ksv.insert("files".into(), files.into_iter().into());

    // And build the outer table for this level.
    Ok(table(super::HFF_DIR, Ecc::INVALID)
        .metadata(ksv.to_bytes::<E>()?)?
        .children(tables)
        .chunks(chunks))
}

fn archive_level<'a, E: ByteOrder>(
    root: &Path,
    path: PathBuf,
    children: Vec<Structure>,
    compression: &impl Fn(&Path) -> Option<u32>,
) -> Result<(Vec<TableBuilder<'a>>, Vec<ChunkDesc<'a>>, Vec<String>)> {
    // The metadata attached to this table will describe the names
    // of the directory and the files which are stored in the chunks.
    let mut ksv = Ksv::new();

    // Insert the path of the root directory for everything we find.
    // This is just the name and not the full path.
    ksv.insert(
        "dir".into(),
        [path.display().to_string()].into_iter().into(),
    );

    // Build children tables and chunks.
    let mut tables = vec![];
    let mut chunks = vec![];
    let mut files = vec![];

    for child in children {
        match child {
            Structure::File(file) => {
                let file: std::path::PathBuf = file.into();
                files.push(file.display().to_string());

                let path = root.join(path.join(&file));
                chunks.push(file_to_chunk(compression, path.into())?);
            }
            Structure::Directory(p, c) => {
                let root = root.join(&path);

                let mut ksv = Ksv::new();
                ksv.insert("dir".into(), [p.display().to_string()].into_iter().into());
                let (t, c, f) = archive_level::<E>(&root, p, c, compression)?;

                // Insert the file names into the metadata.
                ksv.insert("files".into(), f.into_iter().into());

                tables.push(
                    table(super::HFF_DIR, Ecc::INVALID)
                        .children(t.into_iter())
                        .chunks(c.into_iter())
                        .metadata(ksv.to_bytes::<E>()?)?,
                )
            }
        }
    }

    Ok((tables, chunks, files))
}

/// Given a single file, turn it into a table entry.
/// This is used to pack a single file into an archive.
fn archive_single_file<'a, E: ByteOrder>(
    root: &Path,
    file: PathBuf,
    compression: &impl Fn(&Path) -> Option<u32>,
) -> Result<TableBuilder<'a>> {
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
        Ok(hff) => Ok(hff_to_table::<E>(file, hff)?),
        Err(_) => {
            // The file is not an hff, so just pack it into a chunk.
            let file_path: std::path::PathBuf = file_path.into();
            let chunk = file_to_chunk(compression, file_path)?;
            Ok(table(super::HFF_FILE, Ecc::INVALID)
                .chunks([chunk])
                .metadata(ksv.to_bytes::<E>()?)?)
        }
    }
}

/// Given an hff file, convert it into a decomposed table.
fn hff_to_table<'a, E: ByteOrder>(file: PathBuf, hff: Hff<StdReader>) -> Result<TableBuilder<'a>> {
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
