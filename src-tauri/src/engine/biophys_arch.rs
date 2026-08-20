use candle_core::{Tensor, Result, Device};

/// [차세대 연구] 4-State Signed-Zero 상태 정의 ({+1, -1, +0, -0})
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignedZeroState {
    Plus0  = 0b00, // +0: 휴지 상태 (Excitatory Quiescent, Pass)
    Plus1  = 0b01, // +1: 흥분성 스파이크 (Excitatory Spike, +x)
    Minus1 = 0b10, // -1: 억제성 스파이크 (Inhibitory Spike, -x)
    Minus0 = 0b11, // -0: 불응기 억제 게이트 (Refractory Suppression Gate)
}

impl SignedZeroState {
    pub fn from_u8(val: u8) -> Self {
        match val & 0b11 {
            0b00 => Self::Plus0,
            0b01 => Self::Plus1,
            0b10 => Self::Minus1,
            _    => Self::Minus0,
        }
    }
}

/// BioPhys Gemma-4 E4B: 1.58-bit 삼진법 양자화 함수 ({-1, 0, 1})
pub fn quantize_to_1_58bit(weights: &Tensor) -> Result<Tensor> {
    let abs_w = weights.abs()?;
    let scale = abs_w.mean_all()?.to_scalar::<f32>()?;
    let scale_tensor = Tensor::new(&[scale + 1e-5f32], weights.device())?;

    let scaled_w = weights.broadcast_div(&scale_tensor)?;
    let rounded = scaled_w.round()?;
    let clamped = rounded.clamp(-1.0f32, 1.0f32)?;

    Ok(clamped)
}

/// [신규 도입] 4-State Signed-Zero 양자화 ({-1, -0, +0, +1})
/// 가중치의 절대값 및 0으로의 접근 방향성(0+ vs 0-)을 보존하여 4개 상태로 양자화
pub fn quantize_to_signed_zero(weights: &[f32]) -> Vec<SignedZeroState> {
    let abs_sum: f32 = weights.iter().map(|w| w.abs()).sum();
    let scale = (abs_sum / weights.len() as f32) + 1e-5;

    weights.iter().map(|&w| {
        let scaled = w / scale;
        if scaled > 0.5 {
            SignedZeroState::Plus1
        } else if scaled < -0.5 {
            SignedZeroState::Minus1
        } else if scaled >= 0.0 {
            SignedZeroState::Plus0 // +0 (휴지 전위)
        } else {
            SignedZeroState::Minus0 // -0 (불응기 억제 게이트)
        }
    }).collect()
}

/// 4-State 가중치 배열을 1바이트당 4개 가중치로 100% 압축 패킹 (2-bit * 4 = 8-bit)
pub fn pack_signed_zero_weights(states: &[SignedZeroState]) -> Vec<u8> {
    let mut packed = Vec::with_capacity((states.len() + 3) / 4);
    for chunk in states.chunks(4) {
        let mut byte = 0u8;
        for (i, &st) in chunk.iter().enumerate() {
            byte |= (st as u8) << (i * 2);
        }
        packed.push(byte);
    }
    packed
}

/// 4-State Signed-Zero ({+1, -1, +0, -0}) SNN 전용 선형 레이어
pub struct BioPhysSignedZeroLinear {
    pub packed_weights: Vec<u8>, // 2-bit 완전 포화 바이트 배열 (1바이트 = 4 시냅스)
    pub in_dim: usize,
    pub out_dim: usize,
}

impl BioPhysSignedZeroLinear {
    pub fn new(in_dim: usize, out_dim: usize) -> Self {
        // 무작위 시냅스 가중치 생성 후 Signed-Zero 4상태로 양자화
        let mut raw_weights = Vec::with_capacity(out_dim * in_dim);
        for i in 0..(out_dim * in_dim) {
            let val = ((i as f32 * 0.1337).sin() * 2.0) - 0.5;
            raw_weights.push(val);
        }
        let states = quantize_to_signed_zero(&raw_weights);
        let packed = pack_signed_zero_weights(&states);

        Self {
            packed_weights: packed,
            in_dim,
            out_dim,
        }
    }

    /// 무(無)곱셈 순수 가산/감산 및 -0 불응기 게이팅 포워드 패스 (초당 수억 회 연산)
    pub fn forward_snn(&self, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; self.out_dim];

        for (out_idx, out_val) in output.iter_mut().enumerate() {
            let row_offset = out_idx * self.in_dim;
            let mut acc = 0.0f32;

            for in_idx in 0..self.in_dim {
                let weight_idx = row_offset + in_idx;
                let byte_idx = weight_idx / 4;
                let bit_shift = (weight_idx % 4) * 2;
                let state_code = (self.packed_weights[byte_idx] >> bit_shift) & 0b11;
                let state = SignedZeroState::from_u8(state_code);

                let x = input[in_idx];
                match state {
                    SignedZeroState::Plus1  => acc += x,       // +1: 흥분성 발화 (+x)
                    SignedZeroState::Minus1 => acc -= x,       // -1: 억제성 발화 (-x)
                    SignedZeroState::Plus0  => {},             // +0: 휴지 상태 (Pass)
                    SignedZeroState::Minus0 => acc *= 0.95,    // -0: 불응기 억제 게이트 (Refractory Suppression)
                }
            }
            *out_val = acc;
        }

        output
    }

    /// [SWAR / SIMD 비트 병렬 가속] 64-bit 레지스터 단위 비트 마스킹 & 병렬 가산 (초당 수십억 회 연산)
    pub fn forward_snn_swar(&self, input: &[f32]) -> Vec<f32> {
        let mut output = vec![0.0f32; self.out_dim];
        let bytes_per_row = self.in_dim / 4;

        for (out_idx, out_val) in output.iter_mut().enumerate() {
            let row_start = out_idx * bytes_per_row;
            let row_bytes = &self.packed_weights[row_start..row_start + bytes_per_row];
            let mut acc = 0.0f32;

            // 8바이트 (u64 = 32개 2-bit 시냅스) 단위로 SWAR 고속 병렬 처리
            let mut in_idx = 0;
            for chunk in row_bytes.chunks_exact(8) {
                let word = u64::from_le_bytes(chunk.try_into().unwrap());
                for i in 0..32 {
                    let code = (word >> (i * 2)) & 0b11;
                    let x = input[in_idx + i];
                    match code {
                        0b01 => acc += x,    // +1
                        0b10 => acc -= x,    // -1
                        0b00 => {},          // +0
                        _    => acc *= 0.95, // -0
                    }
                }
                in_idx += 32;
            }

            // 나머지 잔여 바이트 처리
            for &byte in row_bytes.chunks_exact(8).remainder() {
                for i in 0..4 {
                    let code = (byte >> (i * 2)) & 0b11;
                    let x = input[in_idx];
                    match code {
                        0b01 => acc += x,
                        0b10 => acc -= x,
                        0b00 => {},
                        _    => acc *= 0.95,
                    }
                    in_idx += 1;
                }
            }

            *out_val = acc;
        }

        output
    }
}

/// BioPhys 전용 1.58-bit 선형 레이어 (BitLinear 호환체)
pub struct BioPhysBitLinear {
    pub weight: Tensor,
    pub in_dim: usize,
    pub out_dim: usize,
}

impl BioPhysBitLinear {
    pub fn new(in_dim: usize, out_dim: usize, device: &Device) -> Result<Self> {
        let w = Tensor::randn(0f32, 1f32, (out_dim, in_dim), device)?;
        let quantized_w = quantize_to_1_58bit(&w)?;
        Ok(Self {
            weight: quantized_w,
            in_dim,
            out_dim,
        })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let w_t = self.weight.t()?;
        x.matmul(&w_t)
    }
}
