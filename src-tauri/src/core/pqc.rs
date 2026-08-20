// Ring-LWE Post-Quantum Cryptography Key Exchange (Eurocrypt 2013)
// 논문: "On Ideal Lattices and Learning with Errors Over Rings" (Lyubashevsky, Peikert, Regev, 2013)

const LATTICE_DIM: usize = 64; // R_q = Z_q[X] / (X^64 + 1)
const LATTICE_Q: i64 = 12289;   // 표준 격자 소수 모듈로

#[derive(Debug, Clone)]
pub struct LatticePolynomial {
    pub coeffs: [i64; LATTICE_DIM],
}

impl LatticePolynomial {
    pub fn zero() -> Self {
        Self { coeffs: [0; LATTICE_DIM] }
    }

    pub fn random_small(seed: u64) -> Self {
        let mut coeffs = [0i64; LATTICE_DIM];
        for (i, c) in coeffs.iter_mut().enumerate() {
            let val = ((seed.wrapping_mul((i as u64) + 1337)) % 5) as i64 - 2; // {-2, -1, 0, 1, 2}
            *c = val;
        }
        Self { coeffs }
    }

    /// 다항식 환 덧셈: A + B mod q
    pub fn add(&self, other: &Self) -> Self {
        let mut res = [0i64; LATTICE_DIM];
        for i in 0..LATTICE_DIM {
            res[i] = (self.coeffs[i] + other.coeffs[i]).rem_euclid(LATTICE_Q);
        }
        Self { coeffs: res }
    }

    /// 다항식 환 곱셈: A * B mod (X^N + 1, q)
    pub fn mul_ring(&self, other: &Self) -> Self {
        let mut res = [0i64; LATTICE_DIM];
        for i in 0..LATTICE_DIM {
            for j in 0..LATTICE_DIM {
                if i + j < LATTICE_DIM {
                    res[i + j] = (res[i + j] + self.coeffs[i] * other.coeffs[j]).rem_euclid(LATTICE_Q);
                } else {
                    // X^N = -1 관계 적용 (Anti-cyclic convolution)
                    let idx = i + j - LATTICE_DIM;
                    res[idx] = (res[idx] - self.coeffs[i] * other.coeffs[j]).rem_euclid(LATTICE_Q);
                }
            }
        }
        Self { coeffs: res }
    }
}

/// 양자 컴퓨터 내성 Ring-LWE 키 교환기
pub struct PostQuantumLatticeKex;

impl PostQuantumLatticeKex {
    /// 공개 키(Public Key) 생성: B = A * S + E mod q
    pub fn generate_keypair(seed: u64) -> (LatticePolynomial, LatticePolynomial) {
        let a = LatticePolynomial::random_small(0xABCD); // 공개 공유 다항식
        let secret = LatticePolynomial::random_small(seed);
        let error = LatticePolynomial::random_small(seed.wrapping_add(1));
        
        let public_b = a.mul_ring(&secret).add(&error);
        (secret, public_b)
    }

    /// 공유 비밀키(Shared Secret) 유도
    pub fn compute_shared_secret(secret: &LatticePolynomial, remote_public: &LatticePolynomial) -> [u8; 32] {
        let shared_poly = remote_public.mul_ring(secret);
        let mut bytes = [0u8; 32];
        for (i, chunk) in shared_poly.coeffs.chunks(2).enumerate() {
            if i < 32 {
                bytes[i] = ((chunk[0] + chunk[1]).rem_euclid(256)) as u8;
            }
        }
        let hash = blake3::hash(&bytes);
        *hash.as_bytes()
    }
}
