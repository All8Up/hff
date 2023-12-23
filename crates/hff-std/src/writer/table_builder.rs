use super::{ChunkDesc, TableDesc};
use crate::{DataSource, Ecc, Error, Result};

#[derive(Debug)]
pub struct TableBuilder<'a> {
    primary: Ecc,
    secondary: Ecc,
    metadata: Option<DataSource<'a>>,
    chunks: Vec<ChunkDesc<'a>>,
    children: Vec<TableBuilder<'a>>,
}

impl<'a> TableBuilder<'a> {
    pub(super) fn new(primary: Ecc, secondary: Ecc) -> Self {
        Self {
            primary,
            secondary,
            metadata: None,
            chunks: vec![],
            children: vec![],
        }
    }

    pub fn metadata<T>(mut self, content: T) -> Result<Self>
    where
        T: TryInto<DataSource<'a>>,
        <T as TryInto<DataSource<'a>>>::Error: std::fmt::Debug,
        Error: From<<T as TryInto<DataSource<'a>>>::Error>,
    {
        self.metadata = Some(content.try_into()?);
        Ok(self)
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TableBuilder<'a>>) -> Self {
        self.children = children.into_iter().collect::<Vec<_>>();
        self
    }

    pub fn chunks(mut self, content: impl IntoIterator<Item = ChunkDesc<'a>>) -> Self {
        self.chunks = content.into_iter().collect::<Vec<_>>();
        self
    }

    pub(super) fn finish(self) -> TableDesc<'a> {
        TableDesc::new(
            self.primary,
            self.secondary,
            self.metadata,
            self.chunks,
            self.children
                .into_iter()
                .map(|desc| desc.finish())
                .collect(),
        )
    }

    pub fn primary(&self) -> Ecc {
        self.primary
    }

    pub fn secondary(&self) -> Ecc {
        self.secondary
    }
}
