#[derive(Encodable, Decodable)]
pub struct EncodedMetadata {
    pub(crate) raw_data: Vec<u8>,
}

impl EncodedMetadata {
    pub fn empty() -> EncodedMetadata {
        EncodedMetadata { raw_data: Vec::new() }
    }

    pub fn uncompressed_metadata(&self) -> &[u8] {
        &self.raw_data
    }

    pub fn compressed_metadata(&self) -> Vec<u8> {
        use snap::write::FrameEncoder;
        use std::io::Write;

        let mut compressed = crate::METADATA_HEADER.to_vec();
        FrameEncoder::new(&mut compressed).write_all(&self.raw_data).unwrap();
        compressed
    }
}
