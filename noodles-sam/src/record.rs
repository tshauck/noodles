//! SAM record.

mod bounds;
mod cigar;
pub mod data;
pub mod fields;
mod name;
mod quality_scores;
mod reference_sequence_name;
mod sequence;

use std::{fmt, io};

use noodles_core::Position;

use self::{bounds::Bounds, fields::Fields};
pub use self::{
    cigar::Cigar, data::Data, name::Name, quality_scores::QualityScores,
    reference_sequence_name::ReferenceSequenceName, sequence::Sequence,
};
use crate::{
    alignment::record::{Flags, MappingQuality},
    Header,
};

/// A SAM record.
#[derive(Clone, Eq, PartialEq)]
pub struct Record {
    pub(crate) buf: Vec<u8>,
    pub(crate) bounds: Bounds,
}

impl Record {
    /// Returns the name.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.name().is_none());
    /// ```
    pub fn name(&self) -> Option<Name<'_>> {
        self.fields().name()
    }

    /// Returns the flags.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam::{self as sam, alignment::record::Flags};
    /// let record = sam::Record::default();
    /// assert_eq!(record.flags()?, Flags::UNMAPPED);
    /// # Ok::<_, std::io::Error>(())
    /// ```
    pub fn flags(&self) -> io::Result<Flags> {
        self.fields().flags().map(Flags::from)
    }

    /// Returns the reference sequence ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let header = sam::Header::default();
    /// let record = sam::Record::default();
    /// assert!(record.reference_sequence_id(&header).is_none());
    /// ```
    pub fn reference_sequence_id(&self, header: &Header) -> Option<io::Result<usize>> {
        self.fields().reference_sequence_id(header)
    }

    /// Returns the reference sequence name.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.reference_sequence_name().is_none());
    /// ```
    pub fn reference_sequence_name(&self) -> Option<ReferenceSequenceName<'_>> {
        self.fields().reference_sequence_name()
    }

    /// Returns the alignment start.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.alignment_start().is_none());
    /// # Ok::<_, std::io::Error>(())
    /// ```
    pub fn alignment_start(&self) -> Option<io::Result<Position>> {
        self.fields().alignment_start()
    }

    /// Returns the mapping quality.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.mapping_quality().is_none());
    /// ```
    pub fn mapping_quality(&self) -> Option<io::Result<MappingQuality>> {
        match self.fields().mapping_quality().transpose() {
            Ok(Some(n)) => MappingQuality::new(n).map(Ok),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }

    /// Returns the CIGAR operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.cigar().is_empty());
    /// ```
    pub fn cigar(&self) -> Cigar<'_> {
        self.fields().cigar()
    }

    /// Returns the mate reference sequence ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let header = sam::Header::default();
    /// let record = sam::Record::default();
    /// assert!(record.mate_reference_sequence_id(&header).is_none());
    /// ```
    pub fn mate_reference_sequence_id(&self, header: &Header) -> Option<io::Result<usize>> {
        self.fields().mate_reference_sequence_id(header)
    }

    /// Returns the mate reference sequence name.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.mate_reference_sequence_name().is_none());
    /// ```
    pub fn mate_reference_sequence_name(&self) -> Option<ReferenceSequenceName<'_>> {
        self.fields().mate_reference_sequence_name()
    }

    /// Returns the mate alignment start.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.mate_alignment_start().is_none());
    /// # Ok::<_, std::io::Error>(())
    /// ```
    pub fn mate_alignment_start(&self) -> Option<io::Result<Position>> {
        self.fields().mate_alignment_start()
    }

    /// Returns the template length.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert_eq!(record.template_length()?, 0);
    /// # Ok::<_, std::io::Error>(())
    /// ```
    pub fn template_length(&self) -> io::Result<i32> {
        self.fields().template_length()
    }

    /// Returns the sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.sequence().is_empty());
    /// ```
    pub fn sequence(&self) -> Sequence<'_> {
        self.fields().sequence()
    }

    /// Returns the quality scores.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.quality_scores().is_empty());
    /// ```
    pub fn quality_scores(&self) -> QualityScores<'_> {
        self.fields().quality_scores()
    }

    /// Returns the data.
    ///
    /// # Examples
    ///
    /// ```
    /// use noodles_sam as sam;
    /// let record = sam::Record::default();
    /// assert!(record.data().is_empty());
    /// ```
    pub fn data(&self) -> Data<'_> {
        self.fields().data()
    }

    fn fields(&self) -> Fields<'_> {
        Fields::new(&self.buf, &self.bounds)
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Record")
            .field("name", &self.name())
            .field("flags", &self.flags())
            .field("reference_sequence_name", &self.reference_sequence_name())
            .field("alignment_start", &self.alignment_start())
            .field("mapping_quality", &self.mapping_quality())
            .field("cigar", &self.cigar())
            .field(
                "mate_reference_sequence_name",
                &self.mate_reference_sequence_name(),
            )
            .field("mate_alignment_start", &self.mate_alignment_start())
            .field("template_length", &self.template_length())
            .field("sequence", &self.sequence())
            .field("quality_scores", &self.quality_scores())
            .field("data", &self.data())
            .finish()
    }
}

impl crate::alignment::Record for Record {
    fn name(&self) -> Option<Box<dyn crate::alignment::record::Name + '_>> {
        let name = self.name()?;
        Some(Box::new(name))
    }

    fn flags(&self) -> io::Result<Flags> {
        self.flags()
    }

    fn reference_sequence_id<'r, 'h: 'r>(
        &'r self,
        header: &'h Header,
    ) -> Option<io::Result<usize>> {
        self.reference_sequence_id(header)
    }

    fn alignment_start(&self) -> Option<io::Result<Position>> {
        self.alignment_start()
    }

    fn mapping_quality(&self) -> Option<io::Result<MappingQuality>> {
        self.mapping_quality()
    }

    fn cigar(&self) -> Box<dyn crate::alignment::record::Cigar + '_> {
        Box::new(self.cigar())
    }

    fn mate_reference_sequence_id<'r, 'h: 'r>(
        &'r self,
        header: &'h Header,
    ) -> Option<io::Result<usize>> {
        self.mate_reference_sequence_id(header)
    }

    fn mate_alignment_start(&self) -> Option<io::Result<Position>> {
        self.mate_alignment_start()
    }

    fn template_length(&self) -> io::Result<i32> {
        self.template_length()
    }

    fn sequence(&self) -> Box<dyn crate::alignment::record::Sequence + '_> {
        Box::new(self.sequence())
    }

    fn quality_scores(&self) -> Box<dyn crate::alignment::record::QualityScores + '_> {
        Box::new(self.quality_scores())
    }

    fn data(&self) -> Box<dyn crate::alignment::record::Data + '_> {
        Box::new(self.data())
    }
}

impl Default for Record {
    fn default() -> Self {
        let buf = b"*4*0255**00**".to_vec();

        let bounds = Bounds {
            name_end: 1,
            flags_end: 2,
            reference_sequence_name_end: 3,
            alignment_start_end: 4,
            mapping_quality_end: 7,
            cigar_end: 8,
            mate_reference_sequence_name_end: 9,
            mate_alignment_start_end: 10,
            template_length_end: 11,
            sequence_end: 12,
            quality_scores_end: 13,
        };

        Self { buf, bounds }
    }
}
