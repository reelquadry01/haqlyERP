// Author: Quadri Atharu

use bytes::Bytes;
use flate2::read::GzipEncoder;
use flate2::Compression;
use std::io::Read;

const MIN_COMPRESS_SIZE: usize = 1024;
const GZIP_LEVEL: u32 = 4;

pub fn compress_response(body: Bytes, accept_encoding: &str) -> (Vec<u8>, Option<&'static str>) {
    if body.len() < MIN_COMPRESS_SIZE {
        return (body.to_vec(), None);
    }

    let encodings: Vec<&str> = accept_encoding
        .split(',')
        .map(|e| e.trim().to_lowercase())
        .collect();

    let prefer_brotli = encodings.iter().any(|e| e == "br" || e.starts_with("br;"));
    let prefer_gzip = encodings.iter().any(|e| e == "gzip" || e.starts_with("gzip;") || e == "x-gzip");

    if prefer_gzip || (!prefer_brotli && encodings.iter().any(|e| e == "*")) {
        return match gzip_compress(&body) {
            Ok(compressed) if compressed.len() < body.len() => (compressed, Some("gzip")),
            _ => (body.to_vec(), None),
        };
    }

    if prefer_brotli {
        return match gzip_compress(&body) {
            Ok(compressed) if compressed.len() < body.len() => (compressed, Some("gzip")),
            _ => (body.to_vec(), None),
        };
    }

    (body.to_vec(), None)
}

fn gzip_compress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = GzipEncoder::new(data, Compression::new(GZIP_LEVEL));
    let mut compressed = Vec::with_capacity(data.len() / 2);
    encoder.read_to_end(&mut compressed)?;
    Ok(compressed)
}

pub struct CompressionLayer;

impl CompressionLayer {
    pub fn new() -> Self {
        CompressionLayer
    }

    pub fn should_compress(content_length: usize, accept_encoding: &str) -> bool {
        if content_length < MIN_COMPRESS_SIZE {
            return false;
        }
        let encodings: Vec<&str> = accept_encoding
            .split(',')
            .map(|e| e.trim().to_lowercase())
            .collect();
        encodings.iter().any(|e| {
            e == "gzip"
                || e.starts_with("gzip;")
                || e == "x-gzip"
                || e == "br"
                || e.starts_with("br;")
                || e == "*"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_compression_small_body() {
        let body = Bytes::from_static(b"small");
        let (compressed, encoding) = compress_response(body, "gzip");
        assert_eq!(encoding, None);
        assert_eq!(compressed.len(), 5);
    }

    #[test]
    fn test_gzip_compression_large_body() {
        let data = "a".repeat(2048);
        let body = Bytes::from(data);
        let (compressed, encoding) = compress_response(body, "gzip");
        assert_eq!(encoding, Some("gzip"));
        assert!(compressed.len() < 2048);
    }

    #[test]
    fn test_no_compression_unsupported_encoding() {
        let data = "a".repeat(2048);
        let body = Bytes::from(data);
        let (compressed, encoding) = compress_response(body, "deflate");
        assert_eq!(encoding, None);
        assert_eq!(compressed.len(), 2048);
    }

    #[test]
    fn test_should_compress() {
        assert!(CompressionLayer::should_compress(2048, "gzip"));
        assert!(CompressionLayer::should_compress(2048, "gzip, deflate"));
        assert!(!CompressionLayer::should_compress(100, "gzip"));
        assert!(!CompressionLayer::should_compress(2048, "identity"));
    }
}
