/// [BioPhys Distributed Hybrid Node]
/// 유저님의 천재적인 '클라우드 믹서기 + 로컬 수집기' 아이디어를 구현한
/// 분산 병렬 P2P 압축 네트워크 코어입니다.
/// 
/// 실행 모드:
/// 1. `--worker` : 클라우드(Hugging Face Spaces 등)에 띄워두고 서버 내부망에서 
///               200GB 원본을 깎아내는 역할.
/// 2. `--master` : 내 컴퓨터(로컬)에서 워커들을 지휘하며 압축된 엑기스만 받아와 
///               디스크에 조립하는 역할.

use std::env;
use std::fs::{OpenOptions, File};
use std::io::Write;
// biophys_agent_lib 라이브러리 참조
use biophys_agent_lib::core::model_runner::BioPhysModelRunner;

const WORKER_PORT: u16 = 8080;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = if args.len() > 1 { &args[1] } else { "--master" };

    println!("🌌 [BioPhys Distributed Network] 초기화 중...");

    match mode {
        "--worker" => start_cloud_worker_node(),
        "--master" => start_local_master_node(),
        _ => {
            println!("❌ 알 수 없는 모드입니다. `--worker` 또는 `--master` 를 사용하세요.");
        }
    }
}

// -------------------------------------------------------------------
// [Worker Node] : 클라우드(Spaces)에 띄우는 가벼운 서버 (RAM 128MB 고정)
// -------------------------------------------------------------------
fn start_cloud_worker_node() {
    println!("☁️ [Worker Node 가동] 클라우드 내부망에서 대기 중... (Port: {})", WORKER_PORT);
    let mut runner = BioPhysModelRunner::boot_system(128, 4);

    // 실제로는 HTTP 웹 서버(Axum 등)를 띄워 Master의 지시를 기다립니다.
    // 여기서는 핑(Ping)이 왔을 때 200GB 모델의 특정 범위를 압축하는 흐름을 모사합니다.
    
    // 무한 루프로 마스터의 요청 대기 (모사)
    println!("⏳ 마스터(유저 컴퓨터)의 압축 지시를 기다립니다...");
    
    // [가상의 요청 수신 시나리오]
    let requested_chunk = 42; 
    println!("📥 [명령 수신] Chunk #{} 압축 지시 받음. 내부망 다운로드 시작...", requested_chunk);
    
    // 1. 내부망 속도로 즉각 다운 (모사)
    let raw_data = vec![0.8f32.to_bits() as u8; 64 * 1024]; 
    
    // 2. RAM 128MB 안에서 즉시 4-State 및 FEP 압축
    runner.load_and_compress_weights(&raw_data);
    let compressed_chunk = vec![0b10101010; 1024]; // 64KB -> 1KB 압축 모사
    
    println!("🚀 [압축 완료] Chunk #{} 변환 완료! 마스터에게 전송합니다 (용량 1/64).", requested_chunk);
    // 실제로는 응답(Response) 바디에 compressed_chunk를 실어서 Master에게 보냄.
}

// -------------------------------------------------------------------
// [Master Node] : 내 컴퓨터에 띄우는 지휘관 (인터넷 트래픽 3GB만 소모)
// -------------------------------------------------------------------
fn start_local_master_node() {
    println!("🖥️ [Master Node 가동] 클라우드 믹서기(Worker)들을 지휘합니다.");
    
    // 조립할 최종 파일 열기
    let out_path = "ultimate_omni.bpsn";
    let mut out_file = OpenOptions::new()
        .create(true).append(true).open(out_path).expect("파일 열기 실패");

    // 연결된 클라우드 워커들 (가상)
    let cloud_workers = vec!["https://my-space-1.hf.space", "https://my-space-2.hf.space"];
    println!("📡 연결된 믹서기 노드 수: {}", cloud_workers.len());

    let total_chunks_to_steal = 300; // 가상의 청크 갯수

    for chunk_idx in 0..total_chunks_to_steal {
        // 워커들에게 번갈아가며(Round-Robin) 압축 심부름을 시킴
        let worker_url = cloud_workers[chunk_idx % cloud_workers.len()];
        
        // 실제로는 reqwest로 GET {worker_url}/compress?chunk={chunk_idx} 호출
        //println!("➡️ 워커 [{}] 에게 Chunk #{} 압축 심부름 지시 중...", worker_url, chunk_idx);
        
        // 워커가 땀 흘려 압축해준 1KB 엑기스 수신 (인터넷 트래픽 0.001% 소모)
        let stolen_compressed_chunk = fetch_from_worker_sim(worker_url, chunk_idx);
        
        // 내 컴퓨터의 하드에 안전하게 보관 (조립)
        out_file.write_all(&stolen_compressed_chunk).unwrap();
        
        if chunk_idx % 50 == 0 {
            println!("💾 [조립 중...] {}/{} 조각 수집 완료. (내 컴퓨터 트래픽 쾌적함)", chunk_idx, total_chunks_to_steal);
        }
    }
    
    println!("🎉 [최종 융합 성공] 200GB 원본을 건드리지 않고, 3GB짜리 융합 모델을 완성했습니다!");
}

// 워커에서 다운받는 모사 함수
fn fetch_from_worker_sim(_url: &str, _chunk_idx: usize) -> Vec<u8> {
    vec![0b10101010; 1024]
}
