use std::path::Path;
use biophys_agent_lib::core::GameVfsOptimizer;

fn main() {
    println!("================================================================================");
    println!("🚀 [BioPhys Game-VFS] 데스티니 2 (153 GB) 차세대 64MB 청크 & 중복 제거 재압축 가동");
    println!("   - AI 모델 주도 GameVfsOptimizer (Zstd-19 & BLAKE3 Deduplication)");
    println!("================================================================================");

    let destiny_dir = Path::new(r"D:\SteamLibrary\steamapps\common\Destiny 2");
    let output_vfs = Path::new(r"D:\SteamLibrary\steamapps\common\Destiny 2_compressed.vfs");

    if !destiny_dir.exists() {
        eprintln!("❌ 데스티니 2 경로가 존재하지 않습니다.");
        return;
    }

    println!("📥 대상 폴더: {}", destiny_dir.display());
    println!("📦 출력 VFS 컨테이너: {}", output_vfs.display());
    println!("⚡ 64MB 청크 단위 Zstd-19 고압축 및 다국어 중복 제거 파이프라인 시작...");

    let optimizer = GameVfsOptimizer::new(6);
    match optimizer.build_vfs_container(destiny_dir, output_vfs) {
        Ok(manifest) => {
            println!("\n================================================================================");
            println!("🎉 [데스티니 2 Game-VFS 재압축 완료]");
            println!("  - 원본 총 용량: {:.2} GB", manifest.total_raw_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("  - 압축된 VFS 용량: {:.2} GB", manifest.total_compressed_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("  - 중복 제거로 비워진 용량: {:.2} GB", manifest.deduplicated_bytes_saved as f64 / 1024.0 / 1024.0 / 1024.0);
            println!("  - 최종 용량 절감률: {:.2}%", manifest.net_savings_ratio_percent);
            println!("================================================================================");
        }
        Err(e) => eprintln!("❌ VFS 압축 실패: {:?}", e),
    }
}
