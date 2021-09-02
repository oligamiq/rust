#[derive(Encodable, Decodable)]
pub struct EncodedMetadata {
    pub raw_data: Vec<u8>,
}

impl EncodedMetadata {
    pub fn new() -> EncodedMetadata {
        EncodedMetadata { raw_data: Vec::new() }
    }
}
