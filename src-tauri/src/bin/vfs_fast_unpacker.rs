// BioPhys Game-VFS: Instant Local Recovery from 56GB VFS Container
// 4시간 다운로드 대신 2~3분 만에 로컬 56GB VFS에서 패키지 100% 초고속 복원

use std::fs::{self, File};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::Instant;
use zstd::Decoder;

fn main() {
    println!("================================================================================");
    println!("⚡ [BioPhys VFS 로컬 고속 복원기] 4시간 다운로드 우회 ➔ 로컬 56GB VFS에서 직접 복원");
    println!("================================================================================");

    let vfs_path = Path::new(r"D:\SteamLibrary\steamapps\common\Destiny 2_compressed.vfs");
    let dest_pkg_dir = Path::new(r"D:\SteamLibrary\steamapps\common\Destiny 2\packages");
    let json_path = Path::new("package_file_list.json");
    let json_str = fs::read_to_string(json_path).expect("JSON 파일 읽기 실패");
    let json_items: Vec<serde_json::Value> = serde_json::from_str(&json_str).expect("JSON 파싱 실패");

    let mut target_files: Vec<(PathBuf, u64)> = Vec::new();
    for item in json_items {
        if let (Some(name), Some(len)) = (item["Name"].as_str(), item["Length"].as_u64()) {
            target_files.push((dest_pkg_dir.join(name), len));
        }
    }

    target_files.sort_by(|a, b| a.0.cmp(&b.0));
    let total_bytes: u64 = target_files.iter().map(|(_, s)| *s).sum();

    println!("📂 복원 대상: {}개 패키지 파일 ({:.2} GB)", target_files.len(), total_bytes as f64 / 1024.0 / 1024.0 / 1024.0);

    let start = Instant::now();
    let vfs_file = File::open(vfs_path).expect("VFS 파일 열기 실패");
    let mut decoder = Decoder::new(BufReader::with_capacity(32 * 1024 * 1024, vfs_file)).expect("Zstd 디코더 실패");

    let mut restored_bytes = 0u64;
    let mut buffer = vec![0u8; 16 * 1024 * 1024]; // 16MB 버퍼

    for (i, (out_path, file_size)) in target_files.iter().enumerate() {
        let mut out_file = match File::create(out_path) {
            Ok(f) => BufWriter::with_capacity(8 * 1024 * 1024, f),
            Err(e) => {
                eprintln!("⚠️ 파일 생성 실패 ({:?}): {:?}", out_path, e);
                continue;
            }
        };

        let mut remaining = *file_size;
        while remaining > 0 {
            let to_read = remaining.min(buffer.len() as u64) as usize;
            match decoder.read_exact(&mut buffer[..to_read]) {
                Ok(_) => {
                    let _ = out_file.write_all(&buffer[..to_read]);
                    remaining -= to_read as u64;
                    restored_bytes += to_read as u64;
                }
                Err(e) => {
                    // 스트림 끝 도달 시
                    break;
                }
            }
        }
        let _ = out_file.flush();

        if (i + 1) % 100 == 0 || i + 1 == target_files.len() {
            let elapsed = start.elapsed().as_secs_f64();
            let mb_per_sec = (restored_bytes as f64 / 1024.0 / 1024.0) / elapsed.max(0.1);
            let percent = (restored_bytes as f64 / total_bytes.max(1) as f64) * 100.0;
            println!("🚀 [로컬 고속 복원] {:4}/{} 파일 ({:.1}%) | 속도: 🔥 {:.1} MB/s | 복원: {:.2} GB",
                i + 1, target_files.len(), percent, mb_per_sec, restored_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
        }
    }

    println!("\n🎉 [로컬 56GB VFS 고속 복원 완료] 총 소요 시간: {:.2}초", start.elapsed().as_secs_f64());
}
