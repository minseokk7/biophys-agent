// Friston Free Energy Principle (FEP) & Active Inference (2006/2009 UCL)
// 논문: "A Free Energy Principle for the Brain" (Karl Friston, J. Physiol-Paris 2006 / Trends Cogn. Sci 2009)

/// 뇌의 변분 자유 에너지 (Variational Free Energy, F) 계산기
/// F = 복잡도(Complexity) - 정확도(Accuracy) = E_q[ln q(s) - ln p(o, s)]
#[derive(Debug, Clone)]
pub struct FreeEnergyController {
    pub prior_belief_entropy: f64,
    pub sensory_prediction_error: f64,
    pub target_refractory_ratio: f64, // -0 억제 게이트 최적 비율
}

impl FreeEnergyController {
    pub fn new() -> Self {
        Self {
            prior_belief_entropy: 1.0,
            sensory_prediction_error: 0.0,
            target_refractory_ratio: 0.25, // 기본 25% 억제 상태
        }
    }

    /// 입력 신호의 불확실성(Entropy)과 예측 오차로부터 변분 자유 에너지 계산
    pub fn update_free_energy(&mut self, input_entropy: f64, prediction_error: f64) -> f64 {
        self.sensory_prediction_error = prediction_error;
        
        // F = KL[q || p] - E[ln p(o|s)] (자유 에너지 수식)
        let complexity = (input_entropy - self.prior_belief_entropy).abs();
        let accuracy = 1.0 / (1.0 + prediction_error);
        let free_energy = complexity + (1.0 - accuracy);

        // 자유 에너지가 높을수록(불확실성이 클수록) -0 억제 게이트 비율을 높여 노이즈 차단
        if free_energy > 1.5 {
            self.target_refractory_ratio = (self.target_refractory_ratio + 0.1).min(0.50);
        } else {
            self.target_refractory_ratio = (self.target_refractory_ratio - 0.05).max(0.15);
        }

        free_energy
    }

    /// 현재 자유 에너지 상태에 따른 SNN 뉴런 억제 임계값 반환
    pub fn get_suppression_threshold(&self) -> f32 {
        self.target_refractory_ratio as f32
    }
}
