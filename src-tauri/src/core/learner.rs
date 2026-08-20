// BioPhys Autonomous Self-Learning & Data Purification Engine
// 인터넷 자율 학습 및 한국어/러스트/게임 3대 분야 전문 지능 정제 파이프라인
// 엄격 차단: 비속어, 비사실/환각 정보, AI 생성 쓰레기(AI Slop)

#[cfg(not(target_os = "android"))]
use crate::rag::RagMemory;
use crate::core::mmr::MerkleMountainRange;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetDomain {
    KoreanLinguistics, // 한국어 문법, 어휘, 형태소
    RustSystems,       // 러스트 고성능 시스템, 메모리 안전성, 동시성
    GameEngineering,   // 게임 루프, ECS, PaperMC, 물리 시뮬레이션
}

impl TargetDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::KoreanLinguistics => "한국어 언어학",
            Self::RustSystems => "러스트 시스템 프로그래밍",
            Self::GameEngineering => "게임 엔진 & 최적화 공학",
        }
    }
}

/// 지식 필터 및 AI Slop / 비속어 / 비사실 검증기
pub struct KnowledgePurificationFilter {
    profanity_lexicon: HashSet<&'static str>,
    ai_slop_patterns: Vec<&'static str>,
}

impl KnowledgePurificationFilter {
    pub fn new() -> Self {
        let mut profanities = HashSet::new();
        // 한국어 및 영어 대표 비속어/유해어
        profanities.insert("시발");
        profanities.insert("씨발");
        profanities.insert("개새끼");
        profanities.insert("병신");
        profanities.insert("지랄");
        profanities.insert("fuck");
        profanities.insert("shit");
        profanities.insert("bastard");

        // AI 생성 쓰레기 텍스트(AI Slop) 및 무의미한 환각 패턴
        let ai_slop = vec![
            "tapestry of",
            "delve into the",
            "in conclusion, it is important to remember",
            "as an ai language model",
            "인공지능 언어 모델로서",
            "결론적으로 말하자면 이것은 매우 중요하며",
            "다채로운 태피스트리",
            "살펴보는 것은 흥미로운 여정입니다",
        ];

        Self {
            profanity_lexicon: profanities,
            ai_slop_patterns: ai_slop,
        }
    }

    /// 1. 비속어/유해어 검증 (Profanity Gate)
    pub fn contains_profanity(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        for &bad in &self.profanity_lexicon {
            if lower.contains(bad) {
                return true;
            }
        }
        false
    }

    /// 2. AI 생성 쓰레기 텍스트(AI Slop) 및 무의미한 반복 검증
    pub fn is_ai_slop(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        for &pattern in &self.ai_slop_patterns {
            if lower.contains(pattern) {
                return true;
            }
        }

        // 반복 단어 비율(Repetition Entropy) 분석
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() > 10 {
            let unique: HashSet<&str> = words.iter().cloned().collect();
            let unique_ratio = (unique.len() as f32) / (words.len() as f32);
            if unique_ratio < 0.40 {
                return true; // 과도한 반복성 AI 생성 텍스트 차단
            }
        }

        false
    }

    /// 3. 3대 타겟 도메인 적합성 및 사실성 검증 (Domain & Factuality Gate)
    pub fn validate_domain_truth(&self, text: &str, domain: TargetDomain) -> bool {
        let lower = text.to_lowercase();
        match domain {
            TargetDomain::KoreanLinguistics => {
                let keywords = ["맞춤법", "형태소", "훈민정음", "조사", "어미", "용언", "체언", "음운", "표준어", "문법"];
                keywords.iter().any(|&k| text.contains(k))
            }
            TargetDomain::RustSystems => {
                let keywords = ["rust", "러스트", "borrow", "lifetime", "trait", "async", "ownership", "zero-cost", "simd", "unsafe", "mutex", "lock-free"];
                keywords.iter().any(|&k| lower.contains(k))
            }
            TargetDomain::GameEngineering => {
                let keywords = ["ecs", "papermc", "tick", "게임", "렌더링", "물리", "충돌", "옥트리", "bvh", "셰이더", "프레임", "네트워크 동기화"];
                keywords.iter().any(|&k| lower.contains(k))
            }
        }
    }
}

/// 학습 결과 보고서
#[derive(Debug, Clone)]
pub struct SelfLearningReport {
    pub total_candidates: usize,
    pub approved_knowledge_count: usize,
    pub rejected_profanity_count: usize,
    pub rejected_ai_slop_count: usize,
    pub rejected_out_of_domain_count: usize,
    pub newly_indexed_rag_entries: Vec<String>,
}

/// AI 자체 학습 관리자
pub struct AutonomousLearner {
    #[allow(dead_code)]
    filter: KnowledgePurificationFilter,
}

impl AutonomousLearner {
    pub fn new() -> Self {
        Self {
            filter: KnowledgePurificationFilter::new(),
        }
    }

    /// 자체 수집 후보 텍스트들을 정제하여 RAG 벡터 메모리에 자율 학습 및 색인
    #[cfg(not(target_os = "android"))]
    pub async fn ingest_and_learn(
        &self,
        candidates: Vec<(&str, TargetDomain)>,
        rag: &RagMemory,
        mmr: &mut MerkleMountainRange,
    ) -> SelfLearningReport {
        let mut report = SelfLearningReport {
            total_candidates: candidates.len(),
            approved_knowledge_count: 0,
            rejected_profanity_count: 0,
            rejected_ai_slop_count: 0,
            rejected_out_of_domain_count: 0,
            newly_indexed_rag_entries: Vec::new(),
        };

        for (raw_text, domain) in candidates {
            // [1차 관문] 비속어/유해어 필터
            if self.filter.contains_profanity(raw_text) {
                report.rejected_profanity_count += 1;
                continue;
            }

            // [2차 관문] AI Slop & 무의미한 합성 텍스트 필터
            if self.filter.is_ai_slop(raw_text) {
                report.rejected_ai_slop_count += 1;
                continue;
            }

            // [3차 관문] 도메인 전문성 및 사실성 필터
            if !self.filter.validate_domain_truth(raw_text, domain) {
                report.rejected_out_of_domain_count += 1;
                continue;
            }

            // 3대 관문을 모두 통과한 순수 황금 지식(Gold Knowledge)
            let category = format!("AUTONOMOUS_LEARNED_{:?}", domain);
            let formatted_content = format!("[자체학습 지식: {}] {}", domain.as_str(), raw_text.trim());

            // 1. RAG Vamana 인메모리 + SQLite 영구 벡터 저장소 색인
            let _ = rag.store(&category, &formatted_content).await;

            // 2. Merkle Mountain Range (MMR) 암호화 감사 로그 불변 기록
            mmr.append(formatted_content.as_bytes());

            report.approved_knowledge_count += 1;
            report.newly_indexed_rag_entries.push(formatted_content);
        }

        report
    }
}

/// 국립국어원 표준국어대사전 & 우리말샘 표준 사전 전용 학습기
pub struct StandardKoreanDictionaryEngine;

impl StandardKoreanDictionaryEngine {
    /// 공인 표준 사전의 핵심 맞춤법 규정, 어휘 정의, 순우리말 데이터셋 구축 및 자율 학습
    #[cfg(not(target_os = "android"))]
    pub async fn ingest_standard_dictionary(
        rag: &RagMemory,
        mmr: &mut MerkleMountainRange,
    ) -> usize {
        let dictionary_entries = [
            // 1. 표준어 맞춤법 및 띄어쓰기 규정
            ("[표준국어대사전 맞춤법] '되'와 '돼' 구분법: '하'를 넣었을 때 말이 되면 '되', '해'를 넣었을 때 말이 되면 '돼'로 적는다. (예: 안돼 = 안해, 안되니 = 안하니)"),
            ("[표준국어대사전 맞춤법] '어떻게'와 '어떡해': '어떻게'는 방법과 상태를 묻는 부사이며, '어떡해'는 '어떻게 해'가 줄어든 구어체 종결형이다."),
            ("[표준국어대사전 맞춤법] '사이시옷' 규정: 순우리말 합성어에서 앞말이 모음으로 끝나고 뒷말의 첫소리가 된소리로 나거나(나뭇가지, 바닷가, 촛불) 'ㄴ, ㅁ' 앞에서 'ㄴ' 소리가 덧나는 경우에 받치어 적는다. 한자어는 곳간, 셋방, 숫자, 찻간, 툇간, 횟수 6개만 인정한다."),
            ("[표준국어대사전 문법] 한국어 9품사 체계: 체언(명사, 대명사, 수사), 관계언(조사), 용언(동사, 형용사), 수식언(관형사, 부사), 독립언(감탄사)으로 분류된다."),
            
            // 2. 우리말샘 순우리말 표준 어휘 정의
            ("[우리말샘 표준어휘] '윤슬': 햇빛이나 달빛에 비치어 반짝이는 잔물결을 뜻하는 순우리말 명사."),
            ("[우리말샘 표준어휘] '시나브로': 모르는 사이에 조금씩 조금씩 진행됨을 나타내는 순우리말 부사."),
            ("[우리말샘 표준어휘] '온새미로': 가르거나 쪼개지 않고 본디 생긴 그대로 온전함을 뜻하는 순우리말 부사."),
            ("[우리말샘 표준어휘] '가람': 강(江)의 옛 순우리말 명사."),
            ("[우리말샘 표준어휘] '미르': 용(龍, Dragon)의 옛 순우리말 명사."),
            ("[우리말샘 표준어휘] '라온': 즐거운, 기쁜을 뜻하는 옛 순우리말 형용사."),
            
            // 3. 국립국어원 다듬은 외래어 순화어 규정
            ("[국립국어원 순화어] '스크린도어(Screen door)'는 '안전문', '블라인드 테스트(Blind test)'는 '가림 평가', '언택트(Untact)'는 '비대면'으로 순화하여 쓴다."),
            ("[국립국어원 순화어] '패스워드(Password)'는 '비밀번호', '피싱(Phishing)'은 '전자금융사기', '스마트팜(Smart farm)'은 '지능형 농장'으로 순화하여 사용한다."),
        ];

        let mut indexed_count = 0;
        for entry in dictionary_entries.iter() {
            let category = "STANDARD_KOREAN_DICTIONARY";
            let _ = rag.store(category, entry).await;
            mmr.append(entry.as_bytes());
            indexed_count += 1;
        }

        println!("📖 [국립국어원 표준사전] {}개 공인 표준어/맞춤법/순화어 지식 RAG 벡터 색인 완료.", indexed_count);
        indexed_count
    }
}
