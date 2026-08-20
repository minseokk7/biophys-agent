// Cuckoo Filter - O(1) Space-Efficient Set Membership with Deletion Support
// 논문: "Cuckoo Filter: Practically Better Than Bloom" (Fan et al., CMU / ACM CoNEXT 2014)

const BUCKET_SIZE: usize = 4;
const MAX_KICKS: usize = 500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fingerprint(pub u8);

#[derive(Clone, Debug)]
pub struct Bucket {
    pub slots: [Option<Fingerprint>; BUCKET_SIZE],
}

impl Bucket {
    pub const fn new() -> Self {
        Self {
            slots: [None; BUCKET_SIZE],
        }
    }

    pub fn insert(&mut self, fp: Fingerprint) -> bool {
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(fp);
                return true;
            }
        }
        false
    }

    pub fn remove(&mut self, fp: Fingerprint) -> bool {
        for slot in self.slots.iter_mut() {
            if *slot == Some(fp) {
                *slot = None;
                return true;
            }
        }
        false
    }

    pub fn contains(&self, fp: Fingerprint) -> bool {
        self.slots.iter().any(|&s| s == Some(fp))
    }
}

/// 뻐꾸기 해싱(Cuckoo Hashing) 기반 확률적 필터 (메모리 극소화 $O(1)$ 중복 차단)
pub struct CuckooFilter {
    buckets: Vec<Bucket>,
    num_buckets: usize,
    count: usize,
}

impl CuckooFilter {
    pub fn new(capacity: usize) -> Self {
        let num_buckets = (capacity / BUCKET_SIZE).max(16).next_power_of_two();
        Self {
            buckets: vec![Bucket::new(); num_buckets],
            num_buckets,
            count: 0,
        }
    }

    fn fingerprint(data: &[u8]) -> Fingerprint {
        let hash = blake3::hash(data);
        let byte = hash.as_bytes()[0];
        // 0은 빈 슬롯과 혼동 방지 위해 1로 매핑
        Fingerprint(if byte == 0 { 1 } else { byte })
    }

    fn get_indices(&self, data: &[u8], fp: Fingerprint) -> (usize, usize) {
        let hash = blake3::hash(data);
        let h1 = (u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap()) as usize) % self.num_buckets;
        
        let fp_hash = blake3::hash(&[fp.0]);
        let fp_offset = u64::from_le_bytes(fp_hash.as_bytes()[..8].try_into().unwrap()) as usize;
        let h2 = (h1 ^ fp_offset) % self.num_buckets;
        (h1, h2)
    }

    /// 데이터 삽입 (뻐꾸기 킥-아웃 알고리즘 수행)
    pub fn insert(&mut self, data: &[u8]) -> bool {
        let fp = Self::fingerprint(data);
        let (i1, i2) = self.get_indices(data, fp);

        if self.buckets[i1].insert(fp) || self.buckets[i2].insert(fp) {
            self.count += 1;
            return true;
        }

        // 두 버킷이 모두 찼을 경우: 임의의 슬롯을 밀어내며(Kick-out) 연쇄 재배치
        let mut cur_idx = if fp.0 % 2 == 0 { i1 } else { i2 };
        let mut cur_fp = fp;

        for _ in 0..MAX_KICKS {
            let slot_idx = (cur_fp.0 as usize) % BUCKET_SIZE;
            let kicked_fp = self.buckets[cur_idx].slots[slot_idx].replace(cur_fp).unwrap();

            let fp_hash = blake3::hash(&[kicked_fp.0]);
            let offset = u64::from_le_bytes(fp_hash.as_bytes()[..8].try_into().unwrap()) as usize;
            cur_idx = (cur_idx ^ offset) % self.num_buckets;
            cur_fp = kicked_fp;

            if self.buckets[cur_idx].insert(cur_fp) {
                self.count += 1;
                return true;
            }
        }
        false
    }

    /// 요소 존재 여부 O(1) 확인 (L1 캐시 레벨 속도)
    pub fn contains(&self, data: &[u8]) -> bool {
        let fp = Self::fingerprint(data);
        let (i1, i2) = self.get_indices(data, fp);
        self.buckets[i1].contains(fp) || self.buckets[i2].contains(fp)
    }

    /// 요소 삭제 (블룸 필터와 달리 즉시 삭제 가능)
    pub fn remove(&mut self, data: &[u8]) -> bool {
        let fp = Self::fingerprint(data);
        let (i1, i2) = self.get_indices(data, fp);
        if self.buckets[i1].remove(fp) || self.buckets[i2].remove(fp) {
            self.count = self.count.saturating_sub(1);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }
}
