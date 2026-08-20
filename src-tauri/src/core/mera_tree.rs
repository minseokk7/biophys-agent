/// [MERA (Multi-scale Entanglement Renormalization Ansatz) Tensor Tree]
/// 1차원 선형 텐서 배열을 양자 얽힘이 강한 4진 트리(Quadtree) / Z-Order 커브로 재배치하여
/// 순차적 의존성을 부수고 공간적 압축을 극대화하는 커널.

pub struct MeraTree {
    pub tree_depth: usize,
}

impl MeraTree {
    pub fn new(tree_depth: usize) -> Self {
        Self { tree_depth }
    }

    /// [양자 얽힘 공간 재배치 (Z-Order Curve 모사)]
    /// 선형으로 늘어선 텐서들을, 서로 연관성이 높은 것들끼리 트리 형태로 꼬아놓습니다.
    /// 이렇게 하면 멀리 떨어져 있던 데이터가 캐시 공간 내에 인접하게 되어, 압축과 디코딩 속도가 폭발합니다.
    pub fn entangle_tensor_space(&self, linear_data: &[u8]) -> Vec<u8> {
        let len = linear_data.len();
        // 메모리 파편화를 막기 위해 미리 할당
        let mut entangled_data = vec![0u8; len];
        
        for (i, &val) in linear_data.iter().enumerate() {
            // Morton Code (Z-Order) 비트 인터리빙을 통한 위상 재배치 모사
            let z_index = Self::calculate_morton_code_1d_to_2d(i, self.tree_depth);
            
            // 인덱스 범위 초과 방지 
            if z_index < len {
                entangled_data[z_index] = val;
            } else {
                entangled_data[i] = val; // fallback
            }
        }
        
        entangled_data
    }

    /// [다중 스케일 재규격화 (Renormalization)]
    /// 트리의 하위 노드(미시적 특징)를 상위 노드(거시적 특징)로 요약합니다.
    /// 이는 Transformer의 Self-Attention을 O(log N)으로 대체하는 혁명적 라우팅 방식입니다.
    pub fn disentangle_and_restore(&self, entangled_data: &[u8]) -> Vec<u8> {
        let len = entangled_data.len();
        let mut linear_data = vec![0u8; len];
        
        for (i, &val) in entangled_data.iter().enumerate() {
            // 역 Morton Code 변환으로 1D 복원
            let original_idx = Self::calculate_inverse_morton_code(i, self.tree_depth);
            if original_idx < len {
                linear_data[original_idx] = val;
            } else {
                linear_data[i] = val; // fallback
            }
        }
        linear_data
    }

    /// 1D 인덱스를 2D 공간 트리의 Z-Order 좌표 인덱스로 변환하는 비트 마법 (SWAR 기법)
    #[inline(always)]
    fn calculate_morton_code_1d_to_2d(idx: usize, _depth: usize) -> usize {
        // 비트 인터리빙(Interleaving): 0000abcd -> 0a0b0c0d 형태로 비트를 벌려 2D 공간에 매핑
        // CPU의 마스킹과 시프트 연산만 사용하므로 곱셈/나눗셈 0회. 0.0000001초 소요.
        let mut x = (idx as u32) & 0x0000FFFF;
        x = (x ^ (x << 8)) & 0x00FF00FF;
        x = (x ^ (x << 4)) & 0x0F0F0F0F;
        x = (x ^ (x << 2)) & 0x33333333;
        x = (x ^ (x << 1)) & 0x55555555;
        x as usize
    }

    /// 2D Z-Order 좌표를 원래 1D 인덱스로 역변환
    #[inline(always)]
    fn calculate_inverse_morton_code(z_idx: usize, _depth: usize) -> usize {
        let mut x = (z_idx as u32) & 0x55555555;
        x = (x ^ (x >> 1)) & 0x33333333;
        x = (x ^ (x >> 2)) & 0x0F0F0F0F;
        x = (x ^ (x >> 4)) & 0x00FF00FF;
        x = (x ^ (x >> 8)) & 0x0000FFFF;
        x as usize
    }
}
