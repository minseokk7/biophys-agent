// Conflict-Free Replicated Data Types (CRDT) & Lamport Vector Clocks
// 논문: "Conflict-Free Replicated Data Types" (Shapiro et al., 2011) & Lamport (1978)

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Lamport Vector Clock (인과율 추적용 벡터 클록)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock {
    pub clocks: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// 로컬 노드 클록 증가
    pub fn increment(&mut self, node_id: &str) {
        let count = self.clocks.entry(node_id.to_string()).or_insert(0);
        *count += 1;
    }

    /// 다른 노드의 벡터 클록과 상한선(Join / Suprenum) 병합
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, &remote_clock) in &other.clocks {
            let local_clock = self.clocks.entry(node.clone()).or_insert(0);
            *local_clock = (*local_clock).max(remote_clock);
        }
    }
}

/// LWW-Element-Set CRDT (Last-Write-Wins 무충돌 분산 집합)
/// 수학적 반격자(Semi-Lattice) 구조로 락 없이 100% 무충돌 단조 수렴
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwElementSet<T: std::hash::Hash + Eq + Clone> {
    pub add_set: HashMap<T, u64>,    // 요소 -> 추가 타임스탬프
    pub remove_set: HashMap<T, u64>, // 요소 -> 삭제 타임스탬프
    pub vector_clock: VectorClock,
}

impl<T: std::hash::Hash + Eq + Clone> LwwElementSet<T> {
    pub fn new() -> Self {
        Self {
            add_set: HashMap::new(),
            remove_set: HashMap::new(),
            vector_clock: VectorClock::new(),
        }
    }

    /// 요소 추가 (Add 연산)
    pub fn add(&mut self, element: T, timestamp: u64, node_id: &str) {
        self.vector_clock.increment(node_id);
        let current = self.add_set.entry(element).or_insert(0);
        *current = (*current).max(timestamp);
    }

    /// 요소 삭제 (Remove 연산)
    pub fn remove(&mut self, element: T, timestamp: u64, node_id: &str) {
        self.vector_clock.increment(node_id);
        let current = self.remove_set.entry(element).or_insert(0);
        *current = (*current).max(timestamp);
    }

    /// 집합에 요소가 존재하는지 조회 (Add timestamp > Remove timestamp)
    pub fn contains(&self, element: &T) -> bool {
        let add_time = self.add_set.get(element).copied().unwrap_or(0);
        let remove_time = self.remove_set.get(element).copied().unwrap_or(0);
        add_time > remove_time
    }

    /// 현재 유효한 모든 요소 목록 반환
    pub fn read_all(&self) -> HashSet<T> {
        let mut active = HashSet::new();
        for (item, &add_time) in &self.add_set {
            let remove_time = self.remove_set.get(item).copied().unwrap_or(0);
            if add_time > remove_time {
                active.insert(item.clone());
            }
        }
        active
    }

    /// 두 기기 간 CRDT 상태 무충돌 결합 (Semi-Lattice Join: LWW_A ⊔ LWW_B)
    pub fn merge(&mut self, remote: &LwwElementSet<T>) {
        // Add Set 병합 (최댓값 유지)
        for (item, &remote_time) in &remote.add_set {
            let local_time = self.add_set.entry(item.clone()).or_insert(0);
            *local_time = (*local_time).max(remote_time);
        }

        // Remove Set 병합 (최댓값 유지)
        for (item, &remote_time) in &remote.remove_set {
            let local_time = self.remove_set.entry(item.clone()).or_insert(0);
            *local_time = (*local_time).max(remote_time);
        }

        // 벡터 클록 병합
        self.vector_clock.merge(&remote.vector_clock);
    }
}
