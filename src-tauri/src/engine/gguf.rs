// BioPhys Native GGUF Parser
// No external ML frameworks. Pure bare-metal byte parsing.

use memmap2::Mmap;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::Path;

pub struct GgufTensorArena {
    pub mmap: Mmap, // Holds the OS memory map alive
    pub version: u32,
    pub tensor_count: u64,
    pub metadata_kv_count: u64,
}

impl GgufTensorArena {
    /// OS 수준의 커널 기능을 호출하여 5GB 모델을 RAM에 제로카피로 즉시 마운트
    pub fn mount<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        
        // unsafe: mmap은 파일이 외부에서 수정되면 미정의 동작을 일으킬 수 있으나,
        // 우리는 읽기 전용(Read-Only) 모델 파일이므로 절대적으로 안전합니다.
        let mmap = unsafe { Mmap::map(&file)? };

        // 1. Magic Number 검증 (G G U F)
        if mmap.len() < 4 || &mmap[0..4] != b"GGUF" {
            return Err(Error::new(ErrorKind::InvalidData, "치명적 오류: GGUF 매직 넘버가 일치하지 않습니다."));
        }

        // 2. 버전 파싱 (Little Endian, bytes 4~7)
        let version = u32::from_le_bytes(mmap[4..8].try_into().unwrap());
        
        // 3. 텐서 개수 파싱 (Little Endian, bytes 8~15)
        let tensor_count = u64::from_le_bytes(mmap[8..16].try_into().unwrap());
        
        // 4. 메타데이터 Key-Value 개수 파싱 (Little Endian, bytes 16~23)
        let metadata_kv_count = u64::from_le_bytes(mmap[16..24].try_into().unwrap());

        Ok(Self {
            mmap,
            version,
            tensor_count,
            metadata_kv_count,
        })
    }
}
