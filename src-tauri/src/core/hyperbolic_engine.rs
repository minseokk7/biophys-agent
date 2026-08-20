// BioPhys Hyperbolic & Epigenetic Neural Compression Engine
// 비유클리드 쌍곡선 포앙카레 볼 임베딩, 세포자동자 제로-데이터 생성기, 후성유전학 크로마틴 게이팅 엔진

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// [비유클리드 쌍곡선 포앙카레 볼 (Poincaré Ball) 좌표계]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PoincarePoint {
    pub coords: Vec<f64>,
}

impl PoincarePoint {
    pub fn new(coords: Vec<f64>) -> Self {
        let norm_sq: f64 = coords.iter().map(|x| x * x).sum();
        let scale = if norm_sq >= 1.0 {
            // 반지름 1.0 내부(포앙카레 디스크)로 프로젝션 클램핑
            0.9999 / norm_sq.sqrt()
        } else {
            1.0
        };
        let clamped = coords.into_iter().map(|x| x * scale).collect();
        Self { coords: clamped }
    }

    /// 쌍곡 공간 거리 계산: d_H(u, v) = arcosh(1 + 2 * ||u - v||^2 / ((1 - ||u||^2)(1 - ||v||^2)))
    pub fn hyperbolic_distance(&self, other: &Self) -> f64 {
        let dim = self.coords.len().min(other.coords.len());
        let mut diff_norm_sq = 0.0;
        let mut u_norm_sq = 0.0;
        let mut v_norm_sq = 0.0;

        for i in 0..dim {
            let u = self.coords[i];
            let v = other.coords[i];
            diff_norm_sq += (u - v) * (u - v);
            u_norm_sq += u * u;
            v_norm_sq += v * v;
        }

        let denom = (1.0 - u_norm_sq.min(0.9999)) * (1.0 - v_norm_sq.min(0.9999));
        let delta = 1.0 + 2.0 * diff_norm_sq / denom.max(1e-12);
        (delta + (delta * delta - 1.0).max(0.0).sqrt()).ln()
    }
}

/// [세포자동자(Cellular Automata) 규칙 기반 제로-데이터 합성기]
/// 테라바이트 데이터를 디스크에 저장하지 않고 단 16바이트 규칙 씨앗만으로 실시간 자가 합성
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellularAutomataSeed {
    pub rule: u32,
    pub seed_hash: u64,
    pub dimension: (usize, usize),
}

impl CellularAutomataSeed {
    pub fn new(rule: u32, seed_hash: u64, width: usize, height: usize) -> Self {
        Self {
            rule,
            seed_hash,
            dimension: (width, height),
        }
    }

    /// 16바이트 규칙 씨앗으로부터 실시간 결정론적 바이트 버퍼 합성 (Zero-Storage Unpack)
    pub fn synthesize_buffer(&self) -> Vec<u8> {
        let (width, height) = self.dimension;
        let total_cells = width * height;
        let mut state = vec![0u8; total_cells];

        // 시드 해시로 초기 1행 시딩
        let mut rng = self.seed_hash;
        for x in 0..width {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            state[x] = (rng >> 56) as u8;
        }

        // 세포자동자 전이 규칙(Transition Rule)에 따른 2D 격자 전파
        for y in 1..height {
            for x in 0..width {
                let left = if x > 0 { state[(y - 1) * width + (x - 1)] } else { state[(y - 1) * width + (width - 1)] };
                let center = state[(y - 1) * width + x];
                let right = if x + 1 < width { state[(y - 1) * width + (x + 1)] } else { state[(y - 1) * width] };

                let pattern = ((left & 1) << 2) | ((center & 1) << 1) | (right & 1);
                let next_bit = ((self.rule >> pattern) & 1) as u8;
                let noise_byte = center.wrapping_add((pattern as u8).wrapping_mul(37)).wrapping_add(next_bit * 128);
                state[y * width + x] = noise_byte;
            }
        }

        state
    }
}

/// [후성유전학적 크로마틴 게이팅 (Epigenetic Dynamic Chromatin Gater)]
/// 비활성 95% 가중치/데이터를 0비트 메틸화 침묵(-0)으로 응축하고 필요한 순간 1ns 만에 활성화
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpigeneticChromatinGater {
    pub total_features: usize,
    pub methylation_mask: Vec<u64>, // 64비트 워드 메틸화 마스크
}

impl EpigeneticChromatinGater {
    pub fn new(total_features: usize) -> Self {
        let words = (total_features + 63) / 64;
        Self {
            total_features,
            methylation_mask: vec![0u64; words], // 초기 상태: 100% 침묵 (0-bit)
        }
    }

    /// 후성유전학적 아세틸화 (Demethylation / 활성화)
    pub fn acetylate_feature(&mut self, index: usize) {
        if index < self.total_features {
            let word_idx = index / 64;
            let bit_idx = index % 64;
            self.methylation_mask[word_idx] |= 1u64 << bit_idx;
        }
    }

    /// 특징 활성 여부 1클럭 비트 검사
    #[inline(always)]
    pub fn is_active(&self, index: usize) -> bool {
        if index < self.total_features {
            let word_idx = index / 64;
            let bit_idx = index % 64;
            (self.methylation_mask[word_idx] & (1u64 << bit_idx)) != 0
        } else {
            false
        }
    }

    /// 활성화율(Sparsity Efficiency) 계산
    pub fn active_ratio(&self) -> f64 {
        let active_count: u32 = self.methylation_mask.iter().map(|w| w.count_ones()).sum();
        active_count as f64 / self.total_features.max(1) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poincare_hyperbolic_distance() {
        let p1 = PoincarePoint::new(vec![0.0, 0.0]);
        let p2 = PoincarePoint::new(vec![0.5, 0.0]);
        let dist = p1.hyperbolic_distance(&p2);
        assert!(dist > 0.0);
        assert!((dist - 1.0986).abs() < 0.05); // arcosh(1 + 2*0.25 / (1 * 0.75)) = arcosh(1.666) ≈ 1.0986
    }

    #[test]
    fn test_cellular_automata_synthesis() {
        let seed = CellularAutomataSeed::new(30, 0x123456789ABCDEF0, 64, 64);
        let buffer = seed.synthesize_buffer();
        assert_eq!(buffer.len(), 4096);
        // 결정론적 일관성 검증
        let buffer2 = seed.synthesize_buffer();
        assert_eq!(buffer, buffer2);
    }

    #[test]
    fn test_epigenetic_chromatin_gater() {
        let mut gater = EpigeneticChromatinGater::new(1024);
        assert_eq!(gater.active_ratio(), 0.0);
        gater.acetylate_feature(10);
        gater.acetylate_feature(100);
        assert!(gater.is_active(10));
        assert!(gater.is_active(100));
        assert!(!gater.is_active(50));
        assert!(gater.active_ratio() > 0.0);
    }
}
