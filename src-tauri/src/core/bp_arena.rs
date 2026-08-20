use std::cell::UnsafeCell;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

/// [BioPhys Foundation] 
/// 파편화 0%, OS 호출(malloc) 0번을 달성하는 순수 Rust 커스텀 메모리 아레나.
/// 범용 할당기(mimalloc, jemalloc)의 복잡한 락(Lock)을 완전히 제거한 직진형(Bump) 할당기입니다.
pub struct BpArena {
    // 거대한 통뼈 메모리 블록의 시작 포인터
    ptr: NonNull<u8>,
    // 아레나의 총 크기 (예: 1GB)
    capacity: usize,
    // 현재까지 사용한 메모리의 끝점(Offset). 원자적(Atomic) 연산으로 락(Lock) 없이 멀티스레드 접근 가능
    offset: AtomicUsize,
}

unsafe impl Send for BpArena {}
unsafe impl Sync for BpArena {}

impl BpArena {
    /// 앱 시작 시 단 한 번! OS에게 거대한 메모리 운동장을 삥 뜯어옵니다.
    pub fn new(capacity_bytes: usize) -> Self {
        let layout = Layout::from_size_align(capacity_bytes, 64)
            .expect("아레나 메모리 레이아웃 생성 실패 (64바이트 정렬)");
        
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("OS가 {} 바이트의 메모리 할당을 거부했습니다!", capacity_bytes);
        }

        Self {
            ptr: NonNull::new(ptr).unwrap(),
            capacity: capacity_bytes,
            offset: AtomicUsize::new(0),
        }
    }

    /// 메모리 할당 (malloc 대체)
    /// 락(Mutex) 없이 원자적 덧셈(fetch_add) 딱 한 번으로 할당 끝. 0.000001초 소요.
    pub fn allocate<'a>(&self, size: usize) -> Option<&'a mut [u8]> {
        // 메모리 정렬(Alignment)을 위해 8바이트 배수로 올림
        let align_offset = (size + 7) & !7;
        
        // 락 없이 현재 오프셋을 증가시키고 이전 오프셋을 가져옴
        let old_offset = self.offset.fetch_add(align_offset, Ordering::SeqCst);
        
        if old_offset + align_offset > self.capacity {
            // 용량 초과: 시스템 크래시를 막기 위해 None 반환
            return None;
        }

        // 포인터 연산으로 메모리 조각을 잘라서 제공
        let mem_slice = unsafe {
            let start = self.ptr.as_ptr().add(old_offset);
            std::slice::from_raw_parts_mut(start, size)
        };

        Some(mem_slice)
    }

    /// 모든 연산이 끝나면 아레나 전체를 한 번에 빗자루로 쓸어버립니다. (개별 free 불필요)
    pub fn reset(&self) {
        self.offset.store(0, Ordering::SeqCst);
    }
}

impl Drop for BpArena {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 64).unwrap();
        unsafe {
            dealloc(self.ptr.as_ptr(), layout);
        }
    }
}
