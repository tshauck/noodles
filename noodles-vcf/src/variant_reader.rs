use std::io;

use super::{Header, Record};

/// A variant format reader.
pub trait VariantReader<R> {
    /// Reads a VCF header.
    fn read_variant_header(&mut self) -> io::Result<Header>;

    /// Returns an iterator over records.
    fn variant_records<'r, 'h: 'r>(
        &'r mut self,
        header: &'h Header,
    ) -> Box<dyn Iterator<Item = io::Result<Record>> + 'r>;
}
