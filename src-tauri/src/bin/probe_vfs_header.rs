use std::fs::File;
use std::io::Read;
use std::path::Path;
use zstd::Decoder;

fn main() {
    let vfs_path = Path::new(r"D:\SteamLibrary\steamapps\common\Destiny 2_compressed.vfs");
    let file = File::open(vfs_path).expect("VFS 열기 실패");
    let mut decoder = Decoder::new(file).expect("Zstd 디코더 실패");
    
    let mut buf = vec![0u8; 1024 * 1024 * 16]; // 16MB
    let n = decoder.read(&mut buf).unwrap_or(0);
    println!("📦 첫 16MB 디코딩 바이트 수: {}", n);
    if n >= 64 {
        println!("  - 첫 64바이트 헥스:");
        for i in 0..4 {
            let slice = &buf[i*16..(i+1)*16];
            println!("    {:02X?}", slice);
        }
        let ascii: String = buf[..64].iter().map(|&b| if b >= 32 && b <= 126 { b as char } else { '.' }).collect();
        println!("  - ASCII: {}", ascii);
    }
}
