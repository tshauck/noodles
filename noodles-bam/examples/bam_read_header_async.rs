//! Prints the header of a BAM file.
//!
//! A BAM file header is a SAM header.
//!
//! The result is similar to the output of `samtools head <src>`.

use std::env;

use noodles_bam as bam;
use noodles_sam as sam;
use tokio::{fs::File, io};

#[tokio::main]
async fn main() -> io::Result<()> {
    let src = env::args().nth(1).expect("missing src");

    let mut reader = File::open(src).await.map(bam::AsyncReader::new)?;
    let raw_header = reader.read_header().await?;

    if raw_header.is_empty() {
        let reference_sequences = reader.read_reference_sequences().await?;

        let header = sam::Header::builder()
            .set_reference_sequences(reference_sequences)
            .build();

        let mut writer = sam::AsyncWriter::new(io::stdout());
        writer.write_header(&header).await?;
    } else {
        print!("{raw_header}");
    }

    Ok(())
}
