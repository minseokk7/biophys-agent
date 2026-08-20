use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use parking_lot::RwLock;

/// 모바일(스마트폰)과 PC를 묶는 P2P 분산망 노드 코어
pub struct SwarmNode {
    pub node_id: String,
    pub is_desktop: bool,
    pub mobile_connected: bool,
}

/// 삼진법 Bit-flip Diff: 두 가중치 배열 간 변경된 인덱스와 새 값만 추출
/// 전체 모델(수백MB)이 아닌 수십KB의 Diff만 전송 — 백서 §1-2 구현
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct BitFlipDiff {
    pub index: usize,
    pub old_val: i8, // {-1, 0, 1}
    pub new_val: i8, // {-1, 0, 1}
}

impl SwarmNode {
    pub fn new() -> Self {
        println!("==================================================");
        println!("🌐 [P2P Swarm] 모바일-PC 이기종 분산망 노드 초기화 중...");
        Self {
            node_id: String::from("BIOPHYS_DESKTOP_9060XT"),
            is_desktop: true,
            mobile_connected: false,
        }
    }

    /// BLAKE3 챌린지-응답 토큰 생성 (암호 인증용)
    /// secret + challenge 를 BLAKE3으로 해싱하여 256-bit 인증 토큰 생성
    pub fn generate_auth_token(challenge: &str) -> String {
        // 공유 비밀키 (양 기기에 동일하게 설정됨)
        const SHARED_SECRET: &str = "BIOPHYS_E4B_1.58BIT_SWARM_SECRET_2026";
        let combined = format!("{}{}", SHARED_SECRET, challenge);
        let hash = blake3::hash(combined.as_bytes());
        hash.to_hex().to_string()
    }

    /// 두 삼진법 가중치 배열 간의 Bit-flip Diff 추출 — 백서 §1-2 핵심 구현
    pub fn extract_bitflip_diff(old_weights: &[i8], new_weights: &[i8]) -> Vec<BitFlipDiff> {
        old_weights.iter().zip(new_weights.iter()).enumerate()
            .filter(|(_, (o, n))| o != n)
            .map(|(idx, (&old_val, &new_val))| BitFlipDiff { index: idx, old_val, new_val })
            .collect()
    }

    /// Bit-flip Diff를 받아 가중치 배열에 패치 적용
    pub fn apply_bitflip_diff(weights: &mut Vec<i8>, diff: &[BitFlipDiff]) {
        for d in diff {
            if d.index < weights.len() {
                weights[d.index] = d.new_val;
            }
        }
    }

    /// 백그라운드에서 24시간 대기하며 스마트폰의 동기화 핑을 수신
    pub async fn start_listener(shared_state: Arc<RwLock<Self>>) {
        // [자동 검색 지원] 모바일 앱이 'BIOPHYS_DISCOVER'를 외치면 PC가 UDP 40506 포트에서 듣고 응답함
        tauri::async_runtime::spawn(async move {
            if let Ok(udp_socket) = tokio::net::UdpSocket::bind("0.0.0.0:40506").await {
                println!("📡 [P2P Swarm] UDP 0.0.0.0:40506 자동 검색 포트 개방 완료.");
                let mut buf = [0u8; 1024];
                loop {
                    if let Ok((len, addr)) = udp_socket.recv_from(&mut buf).await {
                        let msg = String::from_utf8_lossy(&buf[..len]);
                        if msg == "BIOPHYS_DISCOVER" {
                            println!("🔍 [P2P Swarm] 모바일 기기의 자동 검색 요청 감지: {}", addr);
                            let _ = udp_socket.send_to(b"BIOPHYS_PC_HERE", addr).await;
                        }
                    }
                }
            }
        });

        let listener_result = TcpListener::bind("0.0.0.0:40505").await;
        match listener_result {
            Ok(listener) => {
                println!("📡 [P2P Swarm] TCP 0.0.0.0:40505 포트 개방 완료.");
                loop {
                    match listener.accept().await {
                        Ok((mut socket, addr)) => {
                            println!("⚡ [P2P Swarm] 연결 감지: {}", addr);
                            let state_clone = shared_state.clone();
                            tauri::async_runtime::spawn(async move {
                                Self::handle_connection(&mut socket, state_clone).await;
                            });
                        }
                        Err(e) => println!("❌ [P2P Swarm] 소켓 연결 에러: {}", e),
                    }
                }
            }
            Err(e) => println!("❌ [P2P Swarm] 포트 바인딩 실패: {}", e),
        }
    }

    async fn handle_connection(socket: &mut TcpStream, state: Arc<RwLock<Self>>) {
        let mut buffer = [0u8; 4096];
        if let Ok(bytes_read) = socket.read(&mut buffer).await {
            if bytes_read == 0 { return; }
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);

            // [BLAKE3 챌린지-응답 인증] — 플레인텍스트 매칭에서 암호 해시 검증으로 업그레이드
            // 요청 형식: "BIOPHYS_AUTH:<challenge>:<token>"
            if request.starts_with("BIOPHYS_AUTH:") {
                let parts: Vec<&str> = request.trim().splitn(3, ':').collect();
                if parts.len() == 3 {
                    let challenge = parts[1];
                    let received_token = parts[2];
                    let expected_token = Self::generate_auth_token(challenge);

                    if received_token == expected_token {
                        println!("🔐 [P2P Swarm] BLAKE3 인증 통과. 모바일 기기 확인 완료.");
                        {
                            let mut s = state.write();
                            s.mobile_connected = true;
                        }
                        let sync_response = "SYNC_ACK:데스크탑 메인 뇌 가중치 패치 준비 완료";
                        let _ = socket.write_all(sync_response.as_bytes()).await;
                        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        {
                            let mut s = state.write();
                            s.mobile_connected = false;
                        }
                    } else {
                        println!("🚫 [P2P Swarm] BLAKE3 인증 실패. 악성 기기 차단.");
                        let _ = socket.write_all(b"ERROR:INVALID_AUTH_TOKEN").await;
                    }
                }
            } else {
                let _ = socket.write_all(b"ERROR:UNAUTHORIZED_NODE").await;
            }
        }
    }
}
