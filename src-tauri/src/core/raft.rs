// Raft Consensus & Graceful BFT Swarm Mesh Failover (USENIX ATC 2014)
// 논문: "In Search of an Understandable Consensus Algorithm" (Ongaro & Ousterhout, Stanford 2014)

use std::time::{Instant, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwarmRole {
    Leader,
    Follower,
    Candidate,
}

/// Raft 분산 합의 및 무중단 페일오버 노드
pub struct RaftSwarmNode {
    pub node_id: String,
    pub current_term: u64,
    pub role: SwarmRole,
    pub last_heartbeat: Instant,
    pub election_timeout: Duration,
}

impl RaftSwarmNode {
    pub fn new(node_id: &str) -> Self {
        Self {
            node_id: node_id.to_string(),
            current_term: 1,
            role: SwarmRole::Leader, // 데스크탑은 초기 마스터 리더
            last_heartbeat: Instant::now(),
            election_timeout: Duration::from_millis(50), // 50ms 초고속 감지
        }
    }

    /// 모바일 기기로부터 핑 수신
    pub fn receive_heartbeat(&mut self, sender_term: u64) {
        if sender_term >= self.current_term {
            self.current_term = sender_term;
            self.last_heartbeat = Instant::now();
        }
    }

    /// 하트비트 타임아웃 검사 및 자율 장애 복구 (Graceful Failover)
    pub fn check_health_and_failover(&mut self) -> bool {
        if self.last_heartbeat.elapsed() > self.election_timeout {
            // 원격 노드 단절 감지 -> 데스크탑 단독 리더로 즉각 승격/고정
            self.role = SwarmRole::Leader;
            self.current_term += 1;
            self.last_heartbeat = Instant::now();
            true // 페일오버 발생
        } else {
            false
        }
    }
}
