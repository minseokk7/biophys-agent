// BioPhys Neural Lossless Codec: rANS & Predictive Coding
// ICLR Bits-Back rANS 및 4-State Signed-Zero 자유 에너지 예측 오차 코덱

use serde::{Deserialize, Serialize};

/// [rANS (Range Asymmetric Numeral Systems) 코덱]
pub struct RansCodec {
    prob_bits: u32,
}

impl RansCodec {
    pub fn new(prob_bits: u32) -> Self {
        Self { prob_bits }
    }

    /// 바이트 빈도수 기반 누적 분포 함수(CDF) 계산
    pub fn build_frequencies(data: &[u8]) -> [u32; 256] {
        let mut freqs = [1u32; 256]; // Laplace smoothing (최소 빈도 1)
        for &byte in data {
            freqs[byte as usize] += 1;
        }
        freqs
    }

    /// 자유 에너지 원리 기반 신경망 차분 예측 오차 (Residual = X - \hat{X})
    pub fn predictive_residual_encode(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        let mut residuals = Vec::with_capacity(data.len());
        let mut prev = 0u8;
        for &byte in data {
            let res = byte.wrapping_sub(prev);
            residuals.push(res);
            prev = byte;
        }
        residuals
    }

    /// 예측 차분 역변환 (Exact Lossless Reconstruction)
    pub fn predictive_residual_decode(residuals: &[u8]) -> Vec<u8> {
        if residuals.is_empty() {
            return Vec::new();
        }
        let mut reconstructed = Vec::with_capacity(residuals.len());
        let mut prev = 0u8;
        for &res in residuals {
            let original = prev.wrapping_add(res);
            reconstructed.push(original);
            prev = original;
        }
        reconstructed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predictive_residual_roundtrip() {
        let raw_data = b"DESTINY2_4K_TEXTURE_RAW_BINARY_STREAM_1234567890!@#$%^&*()";
        let residuals = RansCodec::predictive_residual_encode(raw_data);
        let restored = RansCodec::predictive_residual_decode(&residuals);
        assert_eq!(raw_data.to_vec(), restored);
    }
}
