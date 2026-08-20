// Merkle Mountain Range (MMR) - Append-Only Cryptographic Audit Log
// 논문: "Merkle Mountain Ranges" (Peter Todd, 2016) / OpenTimestamps

/// MMR 불변 감사 노드
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MmrNode {
    pub position: usize,
    pub hash: [u8; 32],
}

/// Append-Only Merkle Mountain Range (추가 전용 머클 산맥 트리)
/// $O(\log N)$ 크기만의 가벼운 증명으로 시스템 무결성 증명
pub struct MerkleMountainRange {
    pub nodes: Vec<MmrNode>,
    pub count: usize,
}

impl MerkleMountainRange {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            count: 0,
        }
    }

    /// 감사 로그(보안 이벤트, P2P 패치 등)를 머클 산맥에 추가
    pub fn append(&mut self, data: &[u8]) -> usize {
        let leaf_hash = blake3::hash(data);
        let leaf_pos = self.nodes.len();

        self.nodes.push(MmrNode {
            position: leaf_pos,
            hash: *leaf_hash.as_bytes(),
        });
        self.count += 1;

        // 산맥 봉우리(Peak)들을 상향 병합 (Merge Peaks)
        let mut height = 0;
        let mut pos = leaf_pos;
        while self.is_right_child(pos, height) {
            let left_pos = pos - (1 << (height + 1));
            let left_hash = self.nodes[left_pos].hash;
            let right_hash = self.nodes[pos].hash;

            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&left_hash);
            combined[32..].copy_from_slice(&right_hash);
            let parent_hash = blake3::hash(&combined);

            let parent_pos = self.nodes.len();
            self.nodes.push(MmrNode {
                position: parent_pos,
                hash: *parent_hash.as_bytes(),
            });

            pos = parent_pos;
            height += 1;
        }

        leaf_pos
    }

    fn is_right_child(&self, pos: usize, height: usize) -> bool {
        if pos < (1 << (height + 1)) {
            return false;
        }
        // 이진 트리 구조 검사
        (pos + 2) & (1 << (height + 1)) != 0
    }

    /// 현재 산맥의 루트(Root) 해시 요약값 계산 (전체 시스템의 무결성 지문)
    pub fn get_bagged_root(&self) -> [u8; 32] {
        if self.nodes.is_empty() {
            return [0u8; 32];
        }
        let mut acc = self.nodes[0].hash;
        for node in &self.nodes[1..] {
            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(&acc);
            combined[32..].copy_from_slice(&node.hash);
            acc = *blake3::hash(&combined).as_bytes();
        }
        acc
    }
}
