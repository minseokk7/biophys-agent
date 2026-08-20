use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zstd::Decoder;

fn main() {
    println!("🔍 [VFS 복구 테스트] Destiny 2_compressed.vfs 스트림 디코딩 시험...");
    let vfs_path = Path::new(r"D:\SteamLibrary\steamapps\common\Destiny 2_compressed.vfs");
    let file = File::open(vfs_path).expect("VFS 파일 열기 실패");
    
    let mut decoder = Decoder::new(file).expect("Zstd 디코더 초기화 실패");
    let mut sample_buf = vec![0u8; 1024 * 1024]; // 1MB
    match decoder.read(&mut sample_buf) {
        Ok(n) => println!("✅ 디코딩 성공! 첫 1MB 블록 정상 복원: {} bytes", n),
        Err(e) => println!("⚠️ 디코딩 에러: {:?}", e),
    }
}
