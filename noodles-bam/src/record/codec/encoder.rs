//! BAM record encoder.

mod cigar;
pub mod data;
mod mapping_quality;
mod quality_scores;
mod read_name;
mod sequence;

pub(crate) use self::{
    data::put_data, mapping_quality::put_mapping_quality, quality_scores::put_quality_scores,
    read_name::put_read_name, sequence::put_sequence,
};

pub use self::cigar::put_cigar;

use std::io;

use bytes::BufMut;
use noodles_core::Position;
use noodles_sam::{self as sam, alignment::Record, record::Cigar};

// § 4.2.1 "BIN field calculation" (2021-06-03): "Note unmapped reads with `POS` 0 (which
// becomes -1 in BAM) therefore use `reg2bin(-1, 0)` which is computed as 4680."
pub(crate) const UNMAPPED_BIN: u16 = 4680;

pub(crate) fn encode<B>(dst: &mut B, header: &sam::Header, record: &Record) -> io::Result<()>
where
    B: BufMut,
{
    // ref_id
    put_reference_sequence_id(dst, header, record.reference_sequence_id())?;

    // pos
    put_position(dst, record.alignment_start())?;

    put_l_read_name(dst, record.read_name())?;

    // mapq
    put_mapping_quality(dst, record.mapping_quality());

    // bin
    put_bin(dst, record.alignment_start(), record.alignment_end())?;

    // n_cigar_op
    let cigar = overflowing_put_cigar_op_count(dst, header, record)?;

    // flag
    put_flags(dst, record.flags());

    let l_seq = u32::try_from(record.sequence().len())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    dst.put_u32_le(l_seq);

    // next_ref_id
    put_reference_sequence_id(dst, header, record.mate_reference_sequence_id())?;

    // next_pos
    put_position(dst, record.mate_alignment_start())?;

    // tlen
    put_template_length(dst, record.template_length());

    put_read_name(dst, record.read_name());

    if let Some(cigar) = &cigar {
        put_cigar(dst, cigar)?;
    } else {
        put_cigar(dst, record.cigar())?;
    }

    let sequence = record.sequence();
    let quality_scores = record.quality_scores();

    // seq
    put_sequence(dst, record.cigar().read_length(), sequence)?;

    // qual
    put_quality_scores(dst, sequence.len(), quality_scores)?;

    put_data(dst, record.data())?;

    if cigar.is_some() {
        data::field::put_cigar(dst, record.cigar())?;
    }

    Ok(())
}

fn put_reference_sequence_id<B>(
    dst: &mut B,
    header: &sam::Header,
    reference_sequence_id: Option<usize>,
) -> io::Result<()>
where
    B: BufMut,
{
    const UNMAPPED: i32 = -1;

    let ref_id = if let Some(id) = reference_sequence_id {
        if id < header.reference_sequences().len() {
            i32::try_from(id).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "invalid reference sequence ID: expected < {}, got {}",
                    header.reference_sequences().len(),
                    id
                ),
            ));
        }
    } else {
        UNMAPPED
    };

    dst.put_i32_le(ref_id);

    Ok(())
}

fn put_position<B>(dst: &mut B, position: Option<Position>) -> io::Result<()>
where
    B: BufMut,
{
    const MISSING: i32 = -1;

    let pos = if let Some(position) = position {
        i32::try_from(usize::from(position) - 1)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
    } else {
        MISSING
    };

    dst.put_i32_le(pos);

    Ok(())
}

fn put_l_read_name<B>(dst: &mut B, read_name: Option<&sam::record::ReadName>) -> io::Result<()>
where
    B: BufMut,
{
    use std::mem;

    let mut read_name_len = read_name
        .map(|name| name.len())
        .unwrap_or(sam::record::read_name::MISSING.len());

    // + NUL terminator
    read_name_len += mem::size_of::<u8>();

    let l_read_name =
        u8::try_from(read_name_len).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    dst.put_u8(l_read_name);

    Ok(())
}

fn put_bin<B>(
    dst: &mut B,
    alignment_start: Option<Position>,
    alignment_end: Option<Position>,
) -> io::Result<()>
where
    B: BufMut,
{
    let bin = match (alignment_start, alignment_end) {
        (Some(start), Some(end)) => region_to_bin(start, end)?,
        _ => UNMAPPED_BIN,
    };

    dst.put_u16_le(bin);

    Ok(())
}

fn overflowing_put_cigar_op_count<B>(
    dst: &mut B,
    header: &sam::Header,
    record: &Record,
) -> io::Result<Option<Cigar>>
where
    B: BufMut,
{
    use sam::record::cigar::{op, Op};

    if let Ok(n_cigar_op) = u16::try_from(record.cigar().len()) {
        dst.put_u16_le(n_cigar_op);
        Ok(None)
    } else {
        dst.put_u16_le(2);

        let k = record.sequence().len();
        let m = record
            .reference_sequence(header)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "missing reference sequence")
            })?
            .map(|(_, rs)| rs.length().get())?;

        Ok(Some(
            [Op::new(op::Kind::SoftClip, k), Op::new(op::Kind::Skip, m)]
                .into_iter()
                .collect(),
        ))
    }
}

fn put_flags<B>(dst: &mut B, flags: sam::record::Flags)
where
    B: BufMut,
{
    let flag = u16::from(flags);
    dst.put_u16_le(flag);
}

fn put_template_length<B>(dst: &mut B, template_length: i32)
where
    B: BufMut,
{
    dst.put_i32_le(template_length);
}

// § 5.3 "C source code for computing bin number and overlapping bins" (2021-06-03)
#[allow(clippy::eq_op)]
pub(crate) fn region_to_bin(alignment_start: Position, alignment_end: Position) -> io::Result<u16> {
    let start = usize::from(alignment_start) - 1;
    let end = usize::from(alignment_end) - 1;

    let bin = if start >> 14 == end >> 14 {
        ((1 << 15) - 1) / 7 + (start >> 14)
    } else if start >> 17 == end >> 17 {
        ((1 << 12) - 1) / 7 + (start >> 17)
    } else if start >> 20 == end >> 20 {
        ((1 << 9) - 1) / 7 + (start >> 20)
    } else if start >> 23 == end >> 23 {
        ((1 << 6) - 1) / 7 + (start >> 23)
    } else if start >> 26 == end >> 26 {
        ((1 << 3) - 1) / 7 + (start >> 26)
    } else {
        0
    };

    u16::try_from(bin).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use sam::{
        header::record::value::{map::ReferenceSequence, Map},
        record::Flags,
    };

    use super::*;

    #[test]
    fn test_encode_with_default_fields() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let header = sam::Header::default();
        let record = Record::default();
        encode(&mut buf, &header, &record)?;

        let expected = [
            0xff, 0xff, 0xff, 0xff, // ref_id = -1
            0xff, 0xff, 0xff, 0xff, // pos = -1
            0x02, // l_read_name = 2
            0xff, // mapq = 255
            0x48, 0x12, // bin = 4680
            0x00, 0x00, // n_cigar_op = 0
            0x04, 0x00, // flag = 4
            0x00, 0x00, 0x00, 0x00, // l_seq = 0
            0xff, 0xff, 0xff, 0xff, // next_ref_id = -1
            0xff, 0xff, 0xff, 0xff, // next_pos = -1
            0x00, 0x00, 0x00, 0x00, // tlen = 0
            0x2a, 0x00, // read_name = "*\x00"
        ];

        assert_eq!(buf, expected);

        Ok(())
    }

    #[test]
    fn test_encode_with_all_fields() -> Result<(), Box<dyn std::error::Error>> {
        use sam::record::{
            cigar::{op, Op},
            data::field::{tag, Value},
            MappingQuality,
        };

        let mut buf = Vec::new();

        let header = sam::Header::builder()
            .add_reference_sequence(
                "sq0".parse()?,
                Map::<ReferenceSequence>::new(NonZeroUsize::try_from(8)?),
            )
            .add_reference_sequence(
                "sq1".parse()?,
                Map::<ReferenceSequence>::new(NonZeroUsize::try_from(13)?),
            )
            .build();

        let record = Record::builder()
            .set_read_name("r0".parse()?)
            .set_flags(Flags::SEGMENTED | Flags::FIRST_SEGMENT)
            .set_reference_sequence_id(1)
            .set_alignment_start(Position::try_from(9)?)
            .set_mapping_quality(MappingQuality::try_from(13)?)
            .set_cigar(
                [Op::new(op::Kind::Match, 3), Op::new(op::Kind::SoftClip, 1)]
                    .into_iter()
                    .collect(),
            )
            .set_mate_reference_sequence_id(1)
            .set_mate_alignment_start(Position::try_from(22)?)
            .set_template_length(144)
            .set_sequence("ACGT".parse()?)
            .set_quality_scores("NDLS".parse()?)
            .set_data(
                [(tag::ALIGNMENT_HIT_COUNT, Value::from(1))]
                    .into_iter()
                    .collect(),
            )
            .build();

        encode(&mut buf, &header, &record)?;

        let expected = [
            0x01, 0x00, 0x00, 0x00, // ref_id = 1
            0x08, 0x00, 0x00, 0x00, // pos = 8
            0x03, // l_read_name = 3
            0x0d, // mapq = 13
            0x49, 0x12, // bin = 4681
            0x02, 0x00, // n_cigar_op = 2
            0x41, 0x00, // flag = 65
            0x04, 0x00, 0x00, 0x00, // l_seq = 4
            0x01, 0x00, 0x00, 0x00, // next_ref_id = 1
            0x15, 0x00, 0x00, 0x00, // next_pos = 21
            0x90, 0x00, 0x00, 0x00, // tlen = 144
            b'r', b'0', 0x00, // read_name = "r0\x00"
            0x30, 0x00, 0x00, 0x00, // cigar[0] = 3M
            0x14, 0x00, 0x00, 0x00, // cigar[1] = 1S
            0x12, 0x48, // seq = ACGT
            0x2d, 0x23, 0x2b, 0x32, // qual = NDLS
            b'N', b'H', b'C', 0x01, // data[0] = NH:i:1
        ];

        assert_eq!(buf, expected);

        Ok(())
    }

    #[test]
    fn test_encode_with_oversized_cigar() -> Result<(), Box<dyn std::error::Error>> {
        use sam::record::{
            cigar::{op::Kind, Op},
            data::field::{tag, Value},
            sequence::Base,
            Cigar, Sequence,
        };

        const BASE_COUNT: usize = 65536;

        const SQ0_LN: NonZeroUsize = match NonZeroUsize::new(131072) {
            Some(n) => n,
            None => unreachable!(),
        };

        let mut buf = Vec::new();

        let header = sam::Header::builder()
            .add_reference_sequence("sq0".parse()?, Map::<ReferenceSequence>::new(SQ0_LN))
            .build();

        let cigar = Cigar::try_from(vec![Op::new(Kind::Match, 1); BASE_COUNT])?;
        let sequence = Sequence::from(vec![Base::A; BASE_COUNT]);

        let record = Record::builder()
            .set_flags(Flags::empty())
            .set_reference_sequence_id(0)
            .set_alignment_start(Position::MIN)
            .set_cigar(cigar)
            .set_sequence(sequence)
            .set_data(
                [(tag::ALIGNMENT_HIT_COUNT, Value::from(1))]
                    .into_iter()
                    .collect(),
            )
            .build();

        encode(&mut buf, &header, &record)?;

        let mut expected = vec![
            0x00, 0x00, 0x00, 0x00, // ref_id = 0
            0x00, 0x00, 0x00, 0x00, // pos = 1
            0x02, // l_read_name = 2
            0xff, // mapq = 255
            0x49, 0x02, // bin = 585
            0x02, 0x00, // n_cigar_op = 2
            0x00, 0x00, // flag = <empty>
            0x00, 0x00, 0x01, 0x00, // l_seq = 65536
            0xff, 0xff, 0xff, 0xff, // next_ref_id = -1
            0xff, 0xff, 0xff, 0xff, // next_pos = -1
            0x00, 0x00, 0x00, 0x00, // tlen = 0
            b'*', 0x00, // read_name = "*\x00"
            0x04, 0x00, 0x10, 0x00, // cigar[0] = 65536S
            0x03, 0x00, 0x20, 0x00, // cigar[1] = 131072N
        ];

        expected.resize(expected.len() + (BASE_COUNT + 1) / 2, 0x11); // seq = [A, ...]
        expected.resize(expected.len() + BASE_COUNT, 0xff); // qual = [0xff, ...]
        expected.extend([b'N', b'H', b'C', 0x01]); // data[0] = NH:i:1

        // data[1] = CG:B,I:...
        expected.extend([b'C', b'G', b'B', b'I', 0x00, 0x00, 0x01, 0x00]);
        expected.extend((0..BASE_COUNT).flat_map(|_| {
            [
                0x10, 0x00, 0x00, 0x00, // 1M
            ]
        }));

        assert_eq!(buf, expected);

        Ok(())
    }

    #[test]
    fn test_region_to_bin() -> Result<(), Box<dyn std::error::Error>> {
        let start = Position::try_from(8)?;
        let end = Position::try_from(13)?;
        assert_eq!(region_to_bin(start, end)?, 4681);

        let start = Position::try_from(63245986)?;
        let end = Position::try_from(63245986)?;
        assert_eq!(region_to_bin(start, end)?, 8541);

        Ok(())
    }
}
