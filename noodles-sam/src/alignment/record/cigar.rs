//! Alignment record CIGAR operations.

pub mod iter;
pub mod op;

use std::io;

pub use self::op::Op;

/// Alignment record CIGAR operations.
pub trait Cigar {
    /// Returns whether there are any operations.
    fn is_empty(&self) -> bool;

    /// Returns the number of operations.
    fn len(&self) -> usize;

    /// Returns an iterator over operations.
    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Op>> + '_>;

    /// Calculates the alignment span over the reference sequence.
    fn alignment_span(&self) -> io::Result<usize> {
        let mut span = 0;

        for result in self.iter() {
            let op = result?;

            if op.kind().consumes_reference() {
                span += op.len();
            }
        }

        Ok(span)
    }

    /// Calculates the read length.
    fn read_length(&self) -> io::Result<usize> {
        let mut length = 0;

        for result in self.iter() {
            let op = result?;

            if op.kind().consumes_read() {
                length += op.len();
            }
        }

        Ok(length)
    }
}

impl<'a> IntoIterator for &'a dyn Cigar {
    type Item = io::Result<Op>;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Cigar for Box<dyn Cigar + '_> {
    fn is_empty(&self) -> bool {
        (**self).is_empty()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Op>> + '_> {
        (**self).iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alignment::record::cigar::op::Kind;

    struct T(Vec<Op>);

    impl Cigar for T {
        fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        fn len(&self) -> usize {
            self.0.len()
        }

        fn iter(&self) -> Box<dyn Iterator<Item = io::Result<Op>> + '_> {
            Box::new(self.0.iter().copied().map(Ok))
        }
    }

    #[test]
    fn test_into_iter() -> io::Result<()> {
        let cigar: &dyn Cigar = &T(vec![Op::new(Kind::Match, 4)]);

        assert_eq!(
            cigar.into_iter().collect::<io::Result<Vec<_>>>()?,
            [Op::new(Kind::Match, 4)]
        );

        Ok(())
    }

    #[test]
    fn test_alignment_span() -> io::Result<()> {
        let cigar: &dyn Cigar = &T(vec![
            Op::new(Kind::Match, 36),
            Op::new(Kind::Deletion, 4),
            Op::new(Kind::SoftClip, 8),
        ]);

        assert_eq!(cigar.alignment_span()?, 40);

        Ok(())
    }

    #[test]
    fn test_read_length() -> io::Result<()> {
        let cigar: &dyn Cigar = &T(vec![
            Op::new(Kind::Match, 36),
            Op::new(Kind::Deletion, 4),
            Op::new(Kind::SoftClip, 8),
        ]);

        assert_eq!(cigar.read_length()?, 44);

        Ok(())
    }
}
