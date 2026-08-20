// LMAX Disruptor Pattern - Cache-Line Padded Lock-Free Ring Buffer
// 논문: "Disruptor: High Performance Alternative to Bounded Queues for Exchange Architecture" (Thompson et al., 2011)

use std::sync::atomic::{AtomicU64, Ordering};

/// CPU 캐시 라인(64 Byte) 정렬을 보장하여 거짓 공유(False Sharing)를 원천 차단하는 시퀀스
#[repr(align(64))]
pub struct PaddedAtomicSequence {
    pub value: AtomicU64,
}

impl PaddedAtomicSequence {
    pub const fn new(initial: u64) -> Self {
        Self {
            value: AtomicU64::new(initial),
        }
    }
}

/// 64바이트 정렬 고성능 락프리 링버퍼 (배열 크기: 2^N)
pub struct DisruptorRingBuffer<T, const CAP: usize> {
    buffer: Box<[Option<T>; CAP]>,
    cursor: PaddedAtomicSequence,
    published: PaddedAtomicSequence,
}

impl<T: Clone + Default, const CAP: usize> DisruptorRingBuffer<T, CAP> {
    pub fn new() -> Self {
        assert!(CAP.is_power_of_two(), "Ring buffer capacity must be a power of 2");
        let initial_vec = vec![None; CAP];
        let boxed_slice = initial_vec.into_boxed_slice();
        let buffer: Box<[Option<T>; CAP]> = match boxed_slice.try_into() {
            Ok(b) => b,
            Err(_) => unreachable!(),
        };

        Self {
            buffer,
            cursor: PaddedAtomicSequence::new(0),
            published: PaddedAtomicSequence::new(0),
        }
    }

    /// 원자적 시퀀스 획득 후 데이터 쓰기 (나노초 단위 지연시간)
    pub fn publish(&mut self, item: T) -> u64 {
        let seq = self.cursor.value.fetch_add(1, Ordering::SeqCst);
        let mask = (CAP - 1) as u64;
        let index = (seq & mask) as usize;

        self.buffer[index] = Some(item);
        self.published.value.store(seq + 1, Ordering::Release);
        seq
    }

    /// 지정된 시퀀스의 데이터 읽기 (락 없이 즉각 접근)
    pub fn get(&self, seq: u64) -> Option<&T> {
        let mask = (CAP - 1) as u64;
        let index = (seq & mask) as usize;
        self.buffer[index].as_ref()
    }

    /// 현재 발행된 최신 시퀀스 번호 조회
    pub fn latest_sequence(&self) -> u64 {
        self.published.value.load(Ordering::Acquire)
    }
}
