use std::collections::HashMap;

/// BPSN(4-상태) 및 FEP(의문점) 철학을 적용한 경량 바이트-레벨 AI 압축 코어
pub struct NeuralCompressor {
    // 동적 문맥 메모리: (이전 2바이트) -> (다음 바이트 예측 빈도)
    context_memory: HashMap<(u8, u8), HashMap<u8, f32>>,
    recent_history: (u8, u8),
    learning_rate: f32,
}

impl NeuralCompressor {
    pub fn new() -> Self {
        Self {
            context_memory: HashMap::new(),
            recent_history: (0, 0),
            learning_rate: 0.1,
        }
    }

    /// 현재 문맥을 바탕으로 다음 바이트를 예측하고, '의문점(Doubt/Entropy)' 가중치를 반환합니다.
    /// 리턴: (예측된 바이트 튜플, 의문점 가중치 0.0 ~ 1.0)
    pub fn predict_with_doubt(&self) -> (Option<u8>, f32) {
        if let Some(predictions) = self.context_memory.get(&self.recent_history) {
            let mut best_byte = 0u8;
            let mut highest_prob = 0.0f32;
            let mut total_mass = 0.0f32;

            for (&byte, &weight) in predictions.iter() {
                total_mass += weight;
                if weight > highest_prob {
                    highest_prob = weight;
                    best_byte = byte;
                }
            }

            if total_mass > 0.0 {
                let confidence = highest_prob / total_mass;
                let doubt_weight = 1.0 - confidence; // 확신이 100%면 의문점은 0.0
                return (Some(best_byte), doubt_weight);
            }
        }
        
        // 문맥을 본 적이 없으면 의문점 100% (완전 불확실)
        (None, 1.0)
    }

    /// 실제 발생한 바이트를 바탕으로 AI의 가중치를 업데이트(학습) 합니다.
    pub fn update_weights(&mut self, actual_byte: u8) {
        let entry = self.context_memory
            .entry(self.recent_history)
            .or_insert_with(HashMap::new);

        // 정답 가중치 증가 (강화)
        let weight = entry.entry(actual_byte).or_insert(0.0);
        *weight += self.learning_rate;

        // 다른 예측들의 가중치는 자연 감소 (망각 - FEP 원리)
        for w in entry.values_mut() {
            *w *= 0.99; 
        }

        // 히스토리 슬라이딩 윈도우 업데이트
        self.recent_history = (self.recent_history.1, actual_byte);
    }

    /// 데이터 청크를 받아 'AI 의문점 가중치' 기반으로 압축합니다.
    /// 예측 성공 + 의문점 낮음 -> 1비트(0) 메타데이터만 저장
    /// 예측 실패 또는 의문점 높음 -> 1비트(1) 플래그 + 8비트 원본 데이터 저장
    pub fn encode_data(&mut self, raw_data: &[u8]) -> Vec<u8> {
        let mut bit_stream = Vec::new();
        let mut current_byte = 0u8;
        let mut bit_idx = 0;

        for &byte in raw_data {
            let (predicted, doubt) = self.predict_with_doubt();
            
            // 의문점이 0.2 이하(80% 이상 확신)이고 예측이 적중했다면! -> 압축
            if doubt < 0.2 && predicted == Some(byte) {
                // push_bit(false)
                bit_idx += 1;
                if bit_idx == 8 {
                    bit_stream.push(current_byte);
                    current_byte = 0;
                    bit_idx = 0;
                }
            } else {
                // push_bit(true)
                current_byte |= 1 << (7 - bit_idx);
                bit_idx += 1;
                if bit_idx == 8 {
                    bit_stream.push(current_byte);
                    current_byte = 0;
                    bit_idx = 0;
                }
                
                // push_byte_raw(byte)
                for i in (0..8).rev() {
                    let bit = (byte >> i) & 1 == 1;
                    if bit {
                        current_byte |= 1 << (7 - bit_idx);
                    }
                    bit_idx += 1;
                    if bit_idx == 8 {
                        bit_stream.push(current_byte);
                        current_byte = 0;
                        bit_idx = 0;
                    }
                }
            }

            // 실시간 학습 진행
            self.update_weights(byte);
        }

        // 남은 비트 패딩
        if bit_idx > 0 {
            bit_stream.push(current_byte);
        }

        bit_stream
    }

    /// 압축된 비트스트림을 받아 AI가 스스로 데이터를 복원(생성)합니다.
    pub fn decode_data(&mut self, compressed: &[u8], original_len: usize) -> Vec<u8> {
        let mut decoded = Vec::with_capacity(original_len);
        let mut byte_idx = 0;
        let mut bit_idx = 0;

        while decoded.len() < original_len {
            if byte_idx >= compressed.len() { break; }
            
            // read_bit (flag)
            let flag = (compressed[byte_idx] >> (7 - bit_idx)) & 1 == 1;
            bit_idx += 1;
            if bit_idx == 8 {
                bit_idx = 0;
                byte_idx += 1;
            }
            
            let actual_byte = if !flag {
                // 플래그 0: 원본 데이터가 없으므로 AI가 과거의 기억(가중치)으로 환각(생성)해냄!
                let (predicted, _) = self.predict_with_doubt();
                predicted.unwrap_or(0)
            } else {
                // 플래그 1: 예측 실패했으므로 기록된 원본 8비트를 읽음
                let mut val = 0u8;
                for i in (0..8).rev() {
                    if byte_idx >= compressed.len() { break; }
                    let bit = (compressed[byte_idx] >> (7 - bit_idx)) & 1 == 1;
                    bit_idx += 1;
                    if bit_idx == 8 {
                        bit_idx = 0;
                        byte_idx += 1;
                    }
                    if bit { val |= 1 << i; }
                }
                val
            };

            decoded.push(actual_byte);
            
            // 디코딩 과정에서도 동일하게 실시간 학습을 진행해야 다음 문맥을 똑같이 예측 가능
            self.update_weights(actual_byte);
        }

        decoded
    }
}
