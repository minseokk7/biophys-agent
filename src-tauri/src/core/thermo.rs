// Landauer Principle & Reversible Computing Engine (2012 Nature Verification)
// 논문: "Experimental verification of Landauer’s principle linking information and thermodynamics" (Bérut et al., Nature 2012)

/// 란다우어 열역학 한계 상수 (실온 300K 기준, Joules/bit)
pub const LANDAUER_LIMIT_300K_JOULES: f64 = 2.87e-21; // k_B * T * ln(2)

/// 4-State Signed-Zero 가역 연산 및 열역학적 엔트로피 보존기
pub struct LandauerReversibleTracker {
    pub total_operations: u64,
    pub reversible_ops: u64,    // +1, -1, +0 대칭 연산 (소멸 엔트로피 0)
    pub refractory_gates: u64,  // -0 억제 게이트 연산
}

impl LandauerReversibleTracker {
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            reversible_ops: 0,
            refractory_gates: 0,
        }
    }

    /// 4-State 연산의 가역성(Reversibility) 추적 및 이론적 소모 열역학 엔트로피 계산
    pub fn record_batch(&mut self, plus_count: usize, minus_count: usize, plus0_count: usize, minus0_count: usize) {
        let batch_total = plus_count + minus_count + plus0_count + minus0_count;
        self.total_operations += batch_total as u64;
        self.reversible_ops += (plus_count + minus_count + plus0_count) as u64;
        self.refractory_gates += minus0_count as u64;
    }

    /// 가역 연산 비율 반환 (대칭적 보존율)
    pub fn reversibility_ratio(&self) -> f64 {
        if self.total_operations == 0 { return 1.0; }
        (self.reversible_ops as f64) / (self.total_operations as f64)
    }

    /// 기존 비가역 FP32 곱셈 대비 열역학적 에너지 발열 절감률 계산
    pub fn theoretical_energy_dissipated_joules(&self) -> f64 {
        // 비가역적으로 억제된 게이트에 대해서만 최소 란다우어 소멸 열 방출
        (self.refractory_gates as f64) * LANDAUER_LIMIT_300K_JOULES
    }
}
