// BioPhys Advanced Multi-Tier Compression Engine (ANS & Zstandard RFC 8878)
// 논문: "Asymmetric numeral systems: entropy coding combining speed of Huffman coding with compression rate of arithmetic coding" (Jarek Duda)
// RFC 8878: Zstandard Compression and The COVER Dictionary Training Algorithm

use std::io::{Read, Write};

/// [1계층] Zstandard (ANS/FSE 기반) 텍스트 및 사전 레코드 압축기
pub struct ZstdRecordCompressor {
    compression_level: i32, // 기본 3 (초고속 및 고압축 균형)
}

impl ZstdRecordCompressor {
    pub fn new(compression_level: i32) -> Self {
        Self { compression_level }
    }

    /// 텍스트 데이터를 Zstandard 바이트 배열로 고속 압축
    pub fn compress_text(&self, text: &str) -> Result<Vec<u8>, std::io::Error> {
        let raw_bytes = text.as_bytes();
        zstd::encode_all(raw_bytes, self.compression_level)
    }

    /// 압축된 Zstandard 바이트를 원본 텍스트로 고속 복원 (Decompression)
    pub fn decompress_text(&self, compressed: &[u8]) -> Result<String, std::io::Error> {
        let decompressed_bytes = zstd::decode_all(compressed)?;
        String::from_utf8(decompressed_bytes).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })
    }
}

/// [2계층] 4-State 2-Bit 비트팩 벡터 압축기 (128차원 FP32 512B -> 32B 압축)
pub struct VectorBitPacker;

impl VectorBitPacker {
    /// 128차원 연속형 부동소수점 임베딩을 32바이트 2비트 사진수(Quaternary)로 93.75% 압축
    pub fn compress_vector(embedding: &[f32]) -> Vec<u8> {
        let mut packed = Vec::with_capacity((embedding.len() + 3) / 4);
        for chunk in embedding.chunks(4) {
            let mut byte = 0u8;
            for (i, &val) in chunk.iter().enumerate() {
                let code = if val > 0.3 {
                    0b01 // +1
                } else if val < -0.3 {
                    0b10 // -1
                } else if val >= 0.0 {
                    0b00 // +0
                } else {
                    0b11 // -0
                };
                byte |= code << (i * 2);
            }
            packed.push(byte);
        }
        packed
    }

    /// 32바이트 압축 벡터를 128차원 근사 삼진법 부동소수점 배열로 즉각 복원
    pub fn decompress_vector(packed: &[u8], target_len: usize) -> Vec<f32> {
        let mut reconstructed = Vec::with_capacity(target_len);
        for &byte in packed {
            for i in 0..4 {
                if reconstructed.len() >= target_len { break; }
                let code = (byte >> (i * 2)) & 0b11;
                let val = match code {
                    0b01 => 1.0f32,
                    0b10 => -1.0f32,
                    0b00 => 0.0f32,
                    _    => -0.0f32,
                };
                reconstructed.push(val);
            }
        }
        reconstructed
    }
}

/// [3계층] SNN 희소 스파이크 활성화 런렝스(RLE) 압축기
pub struct SparseSpikeRleCompressor;

impl SparseSpikeRleCompressor {
    /// 0(+0/-0)이 연속되는 희소 뉴런 스파이크 신호를 RLE로 압축 (80%+ 절감)
    pub fn compress_spikes(spikes: &[i8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut i = 0;
        while i < spikes.len() {
            let current = spikes[i];
            let mut run_len = 1u8;
            while (i + 1) < spikes.len() && spikes[i + 1] == current && run_len < 255 {
                run_len += 1;
                i += 1;
            }
            compressed.push(run_len);
            compressed.push(current as u8);
            i += 1;
        }
        compressed
    }

    /// RLE 압축 바이트를 원본 스파이크 배열로 복원
    pub fn decompress_spikes(compressed: &[u8]) -> Vec<i8> {
        let mut decompressed = Vec::new();
        for chunk in compressed.chunks_exact(2) {
            let run_len = chunk[0];
            let val = chunk[1] as i8;
            for _ in 0..run_len {
                decompressed.push(val);
            }
        }
        decompressed
    }
}

/// [통합 스토리지 압축 인터페이스]
pub struct UnifiedStorageCompression;

impl UnifiedStorageCompression {
    /// 텍스트 + 벡터를 복합 압축하여 메타데이터와 함께 반환
    pub fn pack_rag_record(role: &str, content: &str, embedding: &[f32]) -> (Vec<u8>, Vec<u8>, f64) {
        let zstd = ZstdRecordCompressor::new(3);
        let raw_text_size = (role.len() + content.len()) as f64;
        let raw_vector_size = (embedding.len() * 4) as f64;
        let raw_total = raw_text_size + raw_vector_size;

        let full_text = format!("{}:{}", role, content);
        let compressed_text = zstd.compress_text(&full_text).unwrap_or_else(|_| full_text.as_bytes().to_vec());
        let compressed_vector = VectorBitPacker::compress_vector(embedding);

        let compressed_total = (compressed_text.len() + compressed_vector.len()) as f64;
        let saved_ratio = (1.0 - (compressed_total / raw_total)) * 100.0;

        (compressed_text, compressed_vector, saved_ratio)
    }
}
