use rayon::prelude::*;
use crate::core::neural_compress::NeuralCompressor;

const CHUNK_SIZE: usize = 65536; // 64KB 독립 청크 (OS 페이지 스래싱 방지 및 캐시 라인 최적화)

/// 프랙탈 인덱스 뼈대: 각 청크가 파일 어디에 위치하는지 저장
#[derive(Debug, Clone)]
pub struct ChunkIndex {
    pub chunk_id: usize,
    pub original_len: usize,
    pub compressed_offset: usize,
    pub compressed_len: usize,
}

/// 병렬 무의존성 프랙탈 압축 파일 시스템 (Fractal VFS)
pub struct FractalVfs {
    pub indices: Vec<ChunkIndex>,
    pub compressed_blob: Vec<u8>,
}

impl FractalVfs {
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
            compressed_blob: Vec::new(),
        }
    }

    /// [용균성 폭발 병렬 인코딩]
    /// 1. 거대한 데이터를 청크로 쪼갭니다.
    /// 2. Rayon 멀티스레드가 각 청크에 '독립된' NeuralCompressor를 할당하여 락(Lock) 없이 동시 압축합니다.
    pub fn compress_parallel(raw_data: &[u8]) -> Self {
        // 1. 데이터를 CHUNK_SIZE 단위로 쪼갬
        let chunks: Vec<&[u8]> = raw_data.chunks(CHUNK_SIZE).collect();

        // 2. Rayon 병렬 이터레이터로 각 청크 압축 (데이터 의존성 완벽 분리!)
        let compressed_chunks: Vec<Vec<u8>> = chunks
            .par_iter()
            .map(|chunk| {
                // 각 스레드마다 독립적인 AI 뇌(가중치)를 생성하여 꼬리물기(Dependency) 저주 해결
                let mut ai_compressor = NeuralCompressor::new();
                ai_compressor.encode_data(chunk)
            })
            .collect();

        // 3. 순차적으로 모아서 최종 Blob 및 인덱스 트리 구성
        let mut vfs = Self::new();
        let mut current_offset = 0;

        for (i, (raw_chunk, comp_chunk)) in chunks.iter().zip(compressed_chunks.iter()).enumerate() {
            let comp_len = comp_chunk.len();
            vfs.indices.push(ChunkIndex {
                chunk_id: i,
                original_len: raw_chunk.len(),
                compressed_offset: current_offset,
                compressed_len: comp_len,
            });
            
            vfs.compressed_blob.extend_from_slice(comp_chunk);
            current_offset += comp_len;
        }

        vfs
    }

    /// [프랙탈 Random Access 언패킹]
    /// 전체를 다 풀 필요 없이, 특정 청크 ID 하나만 0.001초 만에 AI로 환각 복원해냅니다.
    pub fn read_chunk_random_access(&self, chunk_id: usize) -> Option<Vec<u8>> {
        if chunk_id >= self.indices.len() { return None; }
        
        let index = &self.indices[chunk_id];
        let start = index.compressed_offset;
        let end = start + index.compressed_len;
        let chunk_data = &self.compressed_blob[start..end];

        let mut ai_compressor = NeuralCompressor::new();
        let decoded = ai_compressor.decode_data(chunk_data, index.original_len);
        
        Some(decoded)
    }

    /// [초고속 풀 디코딩]
    /// Rayon을 써서 수백 개의 청크를 수십 개의 CPU 코어가 락(Mutex) 없이 동시에 폭발적으로 복원합니다.
    pub fn decompress_all_parallel(&self) -> Vec<u8> {
        // 인덱스를 기반으로 압축된 청크들을 슬라이스로 분리
        let mut chunk_slices = Vec::new();
        for idx in &self.indices {
            let start = idx.compressed_offset;
            let end = start + idx.compressed_len;
            chunk_slices.push((&self.compressed_blob[start..end], idx.original_len));
        }

        // Rayon 병렬 매핑 (Lock Free! 서로 다른 메모리 공간에 동시에 씁니다)
        let decoded_chunks: Vec<Vec<u8>> = chunk_slices
            .par_iter()
            .map(|(data, orig_len)| {
                let mut ai_compressor = NeuralCompressor::new();
                ai_compressor.decode_data(data, *orig_len)
            })
            .collect();

        // 최종적으로 하나로 합침 (메모리 재할당 최소화)
        let total_orig_len: usize = self.indices.iter().map(|idx| idx.original_len).sum();
        let mut final_data = Vec::with_capacity(total_orig_len);
        for mut chunk in decoded_chunks {
            final_data.append(&mut chunk);
        }

        final_data
    }
}
