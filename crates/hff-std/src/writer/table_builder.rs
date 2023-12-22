use super::{ChunkDesc, TableDesc};
use crate::{DataSource, Ecc, Error, Result};

#[derive(Debug)]
pub struct TableBuilder {
    primary: Ecc,
    secondary: Ecc,
    metadata: Option<Box<dyn DataSource>>,
    chunks: Vec<ChunkDesc>,
    children: Vec<TableBuilder>,
}

impl TableBuilder {
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
        T: TryInto<Box<dyn DataSource>>,
        <T as TryInto<Box<dyn DataSource>>>::Error: std::fmt::Debug,
        Error: From<<T as TryInto<Box<dyn DataSource>>>::Error>,
    {
        self.metadata = Some(content.try_into()?);
        Ok(self)
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TableBuilder>) -> Self {
        self.children = children.into_iter().collect::<Vec<_>>();
        self
    }

    pub fn chunks(mut self, content: impl IntoIterator<Item = ChunkDesc>) -> Self {
        self.chunks = content.into_iter().collect::<Vec<_>>();
        self
    }

    pub(super) fn finish(self) -> TableDesc {
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
