// Digital Phase-Locked Loop (DPLL) & Kalman Jitter Compensator
// 논문: "Phaselock Techniques" (Gardner, 1966) & "A New Approach to Linear Filtering" (Kalman, 1960)

/// 1차원 1D 칼만 필터 (네트워크 레이턴시 Jitter 노이즈 제거)
#[derive(Debug, Clone)]
pub struct KalmanJitterFilter {
    estimate: f64,    // 추정된 지연시간 (ms)
    error_cov: f64,   // 추정 오차 공분산
    process_noise: f64, // 시스템 프로세스 잡음 (Q)
    meas_noise: f64,    // 측정 잡음 (R)
}

impl KalmanJitterFilter {
    pub fn new(initial_latency_ms: f64) -> Self {
        Self {
            estimate: initial_latency_ms,
            error_cov: 1.0,
            process_noise: 0.05,
            meas_noise: 0.5,
        }
    }

    /// 측정된 Raw 지연시간(Ping)을 입력받아 최적의 칼만 추정치 반환
    pub fn update(&mut self, raw_latency: f64) -> f64 {
        // 1. 시간 갱신 (Time Update / Predict)
        let prior_cov = self.error_cov + self.process_noise;

        // 2. 칼만 이득(Kalman Gain) 계산
        let kalman_gain = prior_cov / (prior_cov + self.meas_noise);

        // 3. 측정 갱신 (Measurement Update / Correct)
        self.estimate += kalman_gain * (raw_latency - self.estimate);
        self.error_cov = (1.0 - kalman_gain) * prior_cov;

        self.estimate
    }
}

/// 광자 시간 결정(Photonic Clock) 위상 고정 루프 (DPLL)
#[derive(Debug)]
pub struct DigitalPhaseLockedLoop {
    pub local_phase: f64,       // 로컬 클록 위상
    pub nominal_freq: f64,      // 기준 주파수 (1000 Hz = 1ms)
    pub kp: f64,                // 비례 이득 (Proportional Gain)
    pub ki: f64,                // 적분 이득 (Integral Gain)
    integral_error: f64,
    pub jitter_filter: KalmanJitterFilter,
}

impl DigitalPhaseLockedLoop {
    pub fn new(nominal_freq_hz: f64) -> Self {
        Self {
            local_phase: 0.0,
            nominal_freq: nominal_freq_hz,
            kp: 0.15,
            ki: 0.01,
            integral_error: 0.0,
            jitter_filter: KalmanJitterFilter::new(1.0),
        }
    }

    /// 원격 기기(모바일)의 틱 수신 시 위상 오차(Phase Error)를 보정하여 클록 위상 동기화
    pub fn synchronize_tick(&mut self, remote_tick: u64, measured_ping_ms: f64) -> f64 {
        // 1. 칼만 필터로 네트워크 지터 제거
        let filtered_ping = self.jitter_filter.update(measured_ping_ms);
        let one_way_delay = filtered_ping / 2.0;

        // 2. 지연시간을 감안한 원격 위상 추정
        let remote_phase_adjusted = (remote_tick as f64) + (one_way_delay * self.nominal_freq / 1000.0);

        // 3. 위상 오차 (Phase Error) 계산
        let phase_error = remote_phase_adjusted - self.local_phase;

        // 4. PI 루프 필터 적용 (위상 오차를 0으로 수렴)
        self.integral_error += phase_error;
        let correction = (self.kp * phase_error) + (self.ki * self.integral_error);

        self.local_phase += correction;
        self.local_phase
    }

    /// 로컬 1ms 틱 전진
    pub fn advance_local_tick(&mut self) -> u64 {
        self.local_phase += 1.0;
        self.local_phase.round() as u64
    }
}
