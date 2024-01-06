use async_std::stream::StreamExt;
use async_std::{
    fs::read_dir,
    path::{Path, PathBuf},
    task::spawn,
};
use hff_std::{Error, Result};

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
}
