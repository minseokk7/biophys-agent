/// [FEP (Free Energy Principle) Entropy Codec]
/// AI가 시냅스 가중치의 흐름을 예측하고, 
/// 예측이 맞으면 0비트로 날리며 틀리면 잔여 오차(Residual)만 2비트 XOR로 코딩하는 프로덕션 커널.

pub struct FepEntropyCodec {
    /// 과거의 가중치 상태를 기억하여 다음을 예측하기 위한 미니 문맥 버퍼 (Markov Chain 모사)
    history_buffer: u8,
    /// 나비효과를 방지하기 위해 강제로 원본을 박아넣는 주기 (I-Frame 앵커)
    anchor_interval: usize,
}

impl FepEntropyCodec {
    pub fn new(anchor_interval: usize) -> Self {
        Self {
            history_buffer: 0,
            anchor_interval,
        }
    }

    /// [실시간 가중치 예측기]
    /// 과거 2개의 상태(4비트)를 바탕으로, 다음에 올 가중치가 무엇일지 AI가 찍어봅니다.
    #[inline(always)]
    fn predict_next(&self) -> u8 {
        // (프로덕션 모사) 과거 패턴이 +1, -1 반복이었다면 다음은 +1일 것이라 추론하는 단순화된 마르코프 룰
        // 실제로는 더 깊은 텐서 가중치 트리가 사용됩니다.
        match self.history_buffer & 0b1111 {
            0b00_01 => 0b00, // +1, -1 이면 -> +1 (00)
            0b10_10 => 0b10, // +0, +0 이면 -> +0 (10)
            0b11_11 => 0b11, // -0, -0 이면 -> -0 (11)
            _ => 0b10,       // 모르면 보통 가중치가 희소하므로 0(+0)으로 찍음
        }
    }

    /// [엔트로피 인코더 (압축)]
    /// BPSN으로 깎아진 2비트 상태 배열을 받아 예측 압축을 수행합니다.
    /// 반환값: (메타데이터 비트 스트림, 잔여 오차 및 앵커 원본 배열)
    pub fn encode_residual_stream(&mut self, bpsn_states: &[u8]) -> (Vec<bool>, Vec<u8>) {
        let mut meta_bits = Vec::with_capacity(bpsn_states.len());
        let mut residuals = Vec::new();

        for (i, &actual_state) in bpsn_states.iter().enumerate() {
            // 주기적으로 I-Frame 앵커 삽입 (나비효과 원천 차단)
            if i % self.anchor_interval == 0 {
                meta_bits.push(true); // 1 = "이건 닻(Anchor)이니 묻지말고 저장된 거 써라"
                residuals.push(actual_state);
                self.update_history(actual_state);
                continue;
            }

            // AI의 뇌피셜 예측
            let predicted = self.predict_next();

            // 예측과 현실이 완벽히 같다면? (의문점 0)
            if predicted == actual_state {
                meta_bits.push(false); // 0비트 메타데이터 1개만 남기고 데이터 증발!
            } else {
                // 예측이 틀렸다면 잔여 오차(Residual) 발생 (XOR 엇갈림)
                meta_bits.push(true); 
                // 원본을 쌩으로 저장하지 않고, 예측값과의 XOR 엇갈림 플래그만 저장 (더 작게 압축 가능)
                let residual_error = actual_state ^ predicted;
                residuals.push(residual_error);
            }

            // 문맥 업데이트
            self.update_history(actual_state);
        }

        (meta_bits, residuals)
    }

    /// [엔트로피 디코더 (환각 복원)]
    /// 메타데이터(0/1)와 소량의 오차 데이터만 가지고 원본 가중치를 100% 무손실 복원합니다.
    pub fn decode_hallucination(&mut self, meta_bits: &[bool], residuals: &[u8]) -> Vec<u8> {
        let mut restored = Vec::with_capacity(meta_bits.len());
        let mut res_idx = 0;

        for (i, &is_error) in meta_bits.iter().enumerate() {
            if i % self.anchor_interval == 0 {
                // 앵커는 그대로 복원
                let actual = residuals[res_idx];
                res_idx += 1;
                restored.push(actual);
                self.update_history(actual);
                continue;
            }

            let predicted = self.predict_next();

            let actual_state = if !is_error {
                // 기적의 순간: 0비트를 보고 물리적 데이터 없이 AI가 가중치를 창조(환각)해냄!
                predicted
            } else {
                // 오차가 났을 경우, 엇갈림(XOR)을 다시 XOR하여 원본을 무손실 복구
                let residual_error = residuals[res_idx];
                res_idx += 1;
                predicted ^ residual_error
            };

            restored.push(actual_state);
            self.update_history(actual_state);
        }

        restored
    }

    /// 상태를 버퍼에 욱여넣음 (과거 2개 기억)
    #[inline(always)]
    fn update_history(&mut self, state: u8) {
        self.history_buffer = (self.history_buffer << 2) | (state & 0b11);
    }
}
