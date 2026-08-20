use biophys_agent_lib::core::BlockCloningEngine;
use std::path::Path;

fn main() {
    let nikke_dir = Path::new(r"D:\NIKKE");
    println!("🔍 [BioPhys Block Cloner] NIKKE 35,000개 파일 간 교차 중복 클러스터 정밀 스캔 시작...");
    let cloner = BlockCloningEngine::new(64 * 1024); // 64KB 클러스터 단위
    match cloner.analyze_and_clone(nikke_dir) {
        Ok(report) => {
            println!("==================================================");
            println!("📊 [NIKKE 파일 간 교차 중복 블록 스캔 결과]");
            println!("  - 스캔된 총 파일: {} 개", report.total_scanned_files);
            println!("  - 원본 총 용량: {:.2} GB", report.total_raw_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("  - 고유 물리 데이터: {:.2} GB", report.unique_physical_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("  - 중복 공유 가능 용량: {:.2} GB", report.shared_cloned_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("  - 실제 블록 클로닝 절감 잠재력: {:.2}%", report.space_savings_percent);
            println!("==================================================");
        }
        Err(e) => eprintln!("❌ 에러: {:?}", e),
    }
}
