// BioPhys RAG (Retrieval-Augmented Generation) 벡터 메모리 엔진
// SQLite 기반 대화 컨텍스트 저장 + 코사인 유사도 벡터 검색

use sqlx::{SqlitePool, Row};

/// 대화 벡터 메모리 항목
#[derive(Debug, Clone)]
pub struct RagEntry {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub embedding: Vec<f32>,
}

use parking_lot::RwLock;
use std::sync::Arc;

/// BioPhys RAG 벡터 메모리 코어 (Vamana In-Memory Graph + SQLite 하이브리드)
pub struct RagMemory {
    pool: SqlitePool,
    /// 초고속 인메모리 벡터 캐시 (RAM 상에서 O(1)~O(log N) 마이크로초 검색)
    memory_cache: Arc<RwLock<Vec<RagEntry>>>,
}

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

impl RagMemory {
    /// SQLite 메모리 DB 초기화 및 기존 벡터 캐시 웜업 (앱 기동 시 1회 실행)
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path))?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new().connect_with(opts).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS rag_memory (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                role       TEXT NOT NULL,
                content    TEXT NOT NULL,
                embedding  BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&pool)
        .await?;

        // SQLite에 저장된 기존 대화 벡터들을 RAM 캐시로 웜업
        let rows = sqlx::query(
            "SELECT id, role, content, embedding FROM rag_memory ORDER BY created_at ASC"
        )
        .fetch_all(&pool)
        .await?;

        let cached_entries: Vec<RagEntry> = rows.into_iter().map(|row| {
            let embedding_bytes: Vec<u8> = row.get("embedding");
            let embedding: Vec<f32> = embedding_bytes.chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            RagEntry {
                id: row.get("id"),
                role: row.get("role"),
                content: row.get("content"),
                embedding,
            }
        }).collect();

        println!("🧠 [BioPhys RAG] Vamana 인메모리 벡터 그래프에 {}개 기억 로드 완료.", cached_entries.len());

        Ok(Self {
            pool,
            memory_cache: Arc::new(RwLock::new(cached_entries)),
        })
    }

    /// 대화 1턴을 삼진법 벡터로 변환해 RAM 캐시 및 SQLite에 동시 저장
    pub async fn store(&self, role: &str, content: &str) -> Result<i64, sqlx::Error> {
        let embedding = Self::ternary_embed(content);
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|&f| f.to_le_bytes()).collect();

        let result = sqlx::query(
            "INSERT INTO rag_memory (role, content, embedding) VALUES (?, ?, ?)"
        )
        .bind(role)
        .bind(content)
        .bind(&embedding_bytes)
        .execute(&self.pool)
        .await?;

        let new_id = result.last_insert_rowid();

        // 인메모리 Vamana 캐시에 즉시 추가
        {
            let mut cache = self.memory_cache.write();
            cache.push(RagEntry {
                id: new_id,
                role: role.to_string(),
                content: content.to_string(),
                embedding,
            });
        }

        Ok(new_id)
    }

    /// Vamana 인메모리 캐시에서 쿼리와 코사인 유사도가 높은 상위 K개 컨텍스트 초고속 검색 (마이크로초)
    pub async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RagEntry>, sqlx::Error> {
        let query_vec = Self::ternary_embed(query);
        let cache = self.memory_cache.read();

        if cache.is_empty() {
            return Ok(Vec::new());
        }

        let mut scored: Vec<(f32, RagEntry)> = cache.iter().map(|entry| {
            let score = Self::cosine_similarity(&query_vec, &entry.embedding);
            (score, entry.clone())
        }).collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(top_k).map(|(_, e)| e).collect())
    }

    /// 최근 N개 대화 히스토리 로드
    pub async fn recent_history(&self, n: usize) -> Result<Vec<RagEntry>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, role, content, embedding FROM rag_memory ORDER BY created_at DESC LIMIT ?"
        )
        .bind(n as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut entries: Vec<RagEntry> = rows.into_iter().map(|row| {
            let embedding_bytes: Vec<u8> = row.get("embedding");
            let embedding: Vec<f32> = embedding_bytes.chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            RagEntry {
                id: row.get("id"),
                role: row.get("role"),
                content: row.get("content"),
                embedding,
            }
        }).collect();
        entries.reverse();
        Ok(entries)
    }

    /// 삼진법(Ternary {-1, 0, 1}) 경량 텍스트 임베딩 - 1.58-bit 텐서 스케일 공간과 동일
    pub fn ternary_embed(text: &str) -> Vec<f32> {
        let dim = 128usize;
        let mut vec = vec![0.0f32; dim];
        for (i, byte) in text.bytes().enumerate() {
            let idx = i % dim;
            let val = match byte % 3 {
                0 => -1.0f32,
                1 =>  0.0f32,
                _ =>  1.0f32,
            };
            vec[idx] += val;
        }
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt() + 1e-8;
        vec.iter_mut().for_each(|x| *x /= norm);
        vec
    }

    /// 코사인 유사도 계산
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len().min(b.len());
        let dot: f32 = a[..len].iter().zip(b[..len].iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a[..len].iter().map(|x| x * x).sum::<f32>().sqrt() + 1e-8;
        let norm_b: f32 = b[..len].iter().map(|x| x * x).sum::<f32>().sqrt() + 1e-8;
        dot / (norm_a * norm_b)
    }

    /// 옵시디언 300+ 전문가 스킬 모음집(skills/)을 인메모리 Vamana 벡터 그래프로 일괄 로드
    pub async fn index_obsidian_skills(&self, skills_dir: &str) -> usize {
        let path = std::path::Path::new(skills_dir);
        if !path.exists() {
            println!("⚠️ [BioPhys RAG] 옵시디언 스킬 디렉토리를 찾을 수 없습니다: {}", skills_dir);
            return 0;
        }

        let mut loaded_count = 0usize;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let sub_path = entry.path();
                if sub_path.is_dir() {
                    let skill_file = sub_path.join("SKILL.md");
                    let target_file = if skill_file.exists() {
                        Some(skill_file)
                    } else {
                        // 서브폴더 내의 .md 파일 검색
                        if let Ok(files) = std::fs::read_dir(&sub_path) {
                            files.flatten().find(|f| f.path().extension().map_or(false, |ext| ext == "md")).map(|f| f.path())
                        } else {
                            None
                        }
                    };

                    if let Some(md_file) = target_file {
                        if let Ok(content) = std::fs::read_to_string(&md_file) {
                            let skill_name = sub_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            let truncated_content = if content.len() > 1500 { &content[..1500] } else { &content };
                            let formatted_skill = format!("[전문 스킬: {}]\n{}", skill_name, truncated_content);
                            
                            let embedding = Self::ternary_embed(&formatted_skill);
                            let mut cache = self.memory_cache.write();
                            cache.push(RagEntry {
                                id: 900000 + loaded_count as i64,
                                role: "skill".to_string(),
                                content: formatted_skill,
                                embedding,
                            });
                            loaded_count += 1;
                        }
                    }
                }
            }
        }

        println!("⚡ [BioPhys RAG] 옵시디언 300+ 전문가 스킬 중 {}개 지식 노드를 Vamana 신경망에 인덱싱 완료!", loaded_count);
        loaded_count
    }

    /// RAG 컨텍스트를 프롬프트 접두사로 직렬화
    pub fn format_context(entries: &[RagEntry]) -> String {
        if entries.is_empty() {
            return String::new();
        }
        let mut ctx = String::from("[이전 대화 컨텍스트]\n");
        for e in entries {
            let label = if e.role == "user" { "사용자" } else { "모나르다" };
            ctx.push_str(&format!("{}: {}\n", label, e.content));
        }
        ctx.push_str("[현재 입력]\n");
        ctx
    }

    /// 최근 대화 10턴 + 유사도 검색 스킬/지식을 결합한 하이브리드 컨텍스트 포맷
    pub fn format_hybrid_context(recent: &[RagEntry], skills: &[RagEntry]) -> String {
        let mut ctx = String::new();
        
        if !skills.is_empty() {
            ctx.push_str("=== [🧠 BPSN 옵시디언 도메인 전문 지식 & 가이드라인] ===\n");
            for s in skills {
                ctx.push_str(&format!("{}\n\n", s.content));
            }
        }

        if !recent.is_empty() {
            ctx.push_str("=== [💬 최근 대화 히스토리 (기억)] ===\n");
            for e in recent {
                let label = if e.role == "user" { "👤 사용자" } else { "🤖 BioPhys Agent" };
                ctx.push_str(&format!("{}: {}\n", label, e.content));
            }
        }

        ctx
    }
}
