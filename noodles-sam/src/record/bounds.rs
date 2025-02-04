use std::ops::{Range, RangeFrom};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Bounds {
    pub(crate) name_end: usize,
    pub(crate) flags_end: usize,
    pub(crate) reference_sequence_name_end: usize,
    pub(crate) alignment_start_end: usize,
    pub(crate) mapping_quality_end: usize,
    pub(crate) cigar_end: usize,
    pub(crate) mate_reference_sequence_name_end: usize,
    pub(crate) mate_alignment_start_end: usize,
    pub(crate) template_length_end: usize,
    pub(crate) sequence_end: usize,
    pub(crate) quality_scores_end: usize,
}

impl Bounds {
    pub fn name_range(&self) -> Range<usize> {
        0..self.name_end
    }

    pub fn flags_range(&self) -> Range<usize> {
        self.name_end..self.flags_end
    }

    pub fn reference_sequence_name_range(&self) -> Range<usize> {
        self.flags_end..self.reference_sequence_name_end
    }

    pub fn alignment_start_range(&self) -> Range<usize> {
        self.reference_sequence_name_end..self.alignment_start_end
    }

    pub fn mapping_quality_range(&self) -> Range<usize> {
        self.alignment_start_end..self.mapping_quality_end
    }

    pub fn cigar_range(&self) -> Range<usize> {
        self.mapping_quality_end..self.cigar_end
    }

    pub fn mate_reference_sequence_name_range(&self) -> Range<usize> {
        self.cigar_end..self.mate_reference_sequence_name_end
    }

    pub fn mate_alignment_start_range(&self) -> Range<usize> {
        self.mate_reference_sequence_name_end..self.mate_alignment_start_end
    }

    pub fn template_length_range(&self) -> Range<usize> {
        self.mate_alignment_start_end..self.template_length_end
    }

    pub fn sequence_range(&self) -> Range<usize> {
        self.template_length_end..self.sequence_end
    }

    pub fn quality_scores_range(&self) -> Range<usize> {
        self.sequence_end..self.quality_scores_end
    }

    pub fn data_range(&self) -> RangeFrom<usize> {
        self.quality_scores_end..
    }
}
