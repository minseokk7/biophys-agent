#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

#[cfg(not(target_os = "android"))]
pub mod engine;
pub mod proxy;
pub mod p2p;
#[cfg(not(target_os = "android"))]
pub mod rag;
#[cfg(not(target_os = "android"))]
pub mod native_tts;
pub mod core;
pub mod app_generator;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(target_os = "android")]
use std::sync::Mutex;
#[cfg(target_os = "android")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(target_os = "android")]
struct AppState {
    pc_ip: Mutex<Option<String>>,
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn send_prompt(prompt: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let pc_ip = {
        let lock = state.pc_ip.lock().unwrap();
        lock.clone()
    };

    let ip = match pc_ip {
        Some(ip) => ip,
        None => return Ok("[P2P 폰 전용 모드] PC를 찾지 못했습니다. '자동 검색'을 먼저 진행해주세요.".into()),
    };

    let mut stream = tokio::net::TcpStream::connect(format!("{}:40505", ip)).await
        .map_err(|e| format!("PC 연결 실패: {}", e))?;

    // BLAKE3 인증 챌린지 
    // 원래 p2p::SwarmNode에서 가져와야 하지만, 안드로이드 빌드에서는 p2p가 제외될 수 있으므로 임시 구현
    let challenge = "MOBILE_PROMPT";
    let shared_secret = "BIOPHYS_E4B_1.58BIT_SWARM_SECRET_2026";
    let combined = format!("{}{}", shared_secret, challenge);
    let token = blake3::hash(combined.as_bytes()).to_hex().to_string();

    let auth_msg = format!("BIOPHYS_AUTH:{}:{}", challenge, token);
    stream.write_all(auth_msg.as_bytes()).await.map_err(|e| e.to_string())?;

    let mut buf = [0u8; 1024];
    let len = stream.read(&mut buf).await.map_err(|e| e.to_string())?;
    let reply = String::from_utf8_lossy(&buf[..len]).to_string();

    // 임시: 원래는 프롬프트 전송 로직이 들어가야 함
    Ok(format!("[P2P 폰 전용 모드] PC 응답: {}", reply))
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn auto_connect_to_pc(state: tauri::State<'_, AppState>) -> Result<String, String> {
    use tokio::net::{UdpSocket, TcpStream};
    use std::time::Duration;

    // 1. UDP 브로드캐스트 (빠르지만 방화벽에 막힐 수 있음)
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
        if socket.set_broadcast(true).is_ok() {
            let _ = socket.send_to(b"BIOPHYS_DISCOVER", "255.255.255.255:40506").await;
            let mut buf = [0u8; 1024];
            if let Ok(Ok((len, addr))) = tokio::time::timeout(Duration::from_millis(500), socket.recv_from(&mut buf)).await {
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg == "BIOPHYS_PC_HERE" {
                    let ip_str = addr.ip().to_string();
                    *state.pc_ip.lock().unwrap() = Some(ip_str.clone());
                    return Ok(format!("PC 연결 성공 (UDP): {}", ip_str));
                }
            }
        }
    }

    // 2. TCP 서브넷 스윕 (가장 확실한 방법, 192.168.0.x 대역을 병렬 스캔)
    let mut tasks = Vec::new();
    for i in 1..=254 {
        let ip = format!("192.168.0.{}", i);
        let task = tauri::async_runtime::spawn(async move {
            let addr = format!("{}:40505", ip);
            if tokio::time::timeout(Duration::from_millis(800), TcpStream::connect(&addr)).await.is_ok() {
                Some(ip)
            } else {
                None
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        if let Ok(Some(found_ip)) = task.await {
            *state.pc_ip.lock().unwrap() = Some(found_ip.clone());
            return Ok(format!("PC 연결 성공 (TCP Scan): {}", found_ip));
        }
    }

    Err("PC를 찾을 수 없습니다. (방화벽이 40505 포트를 막고 있는지 확인하세요)".into())
}

#[cfg(target_os = "android")]
#[tauri::command]
fn get_swarm_status(state: tauri::State<'_, AppState>) -> String {
    let connected = state.pc_ip.lock().unwrap().is_some();
    serde_json::json!({
        "node_id": "BIOPHYS_MOBILE_NODE",
        "is_desktop": false,
        "mobile_connected": connected
    }).to_string()
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn synthesize_audio(_text: String) -> Result<Vec<u8>, String> {
    Ok(Vec::new()) 
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn trigger_autonomous_learning() -> Result<String, String> {
    Ok("모바일 환경에서는 백그라운드 학습이 제한됩니다.".into())
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn scan_installed_games() -> Result<String, String> {
    Ok("[]".into())
}

#[cfg(target_os = "android")]
#[tauri::command]
async fn compress_game_folder(_path: String) -> Result<String, String> {
    Ok("모바일 압축 엔진 연결 대기 중".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "android")]
    {
        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .manage(AppState { pc_ip: std::sync::Mutex::new(None) })
            .invoke_handler(tauri::generate_handler![
                greet,
                send_prompt,
                auto_connect_to_pc,
                get_swarm_status,
                synthesize_audio,
                trigger_autonomous_learning,
                scan_installed_games,
                compress_game_folder
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
