//! CRAM data container and fields.

pub(crate) mod builder;
pub(crate) mod compression_header;
pub(crate) mod slice;

pub use self::{compression_header::CompressionHeader, slice::Slice};

pub(crate) use self::builder::Builder;

/// A CRAM data container.
pub struct DataContainer {
    compression_header: CompressionHeader,
    slices: Vec<Slice>,
}

impl DataContainer {
    pub(crate) fn builder(record_counter: i64) -> Builder {
        Builder::new(record_counter)
    }

    pub(crate) fn new(compression_header: CompressionHeader, slices: Vec<Slice>) -> Self {
        Self {
            compression_header,
            slices,
        }
    }

    /// Returns the compression header.
    pub fn compression_header(&self) -> &CompressionHeader {
        &self.compression_header
    }

    /// Returns a list of slices.
    pub fn slices(&self) -> &[Slice] {
        &self.slices
    }
}
