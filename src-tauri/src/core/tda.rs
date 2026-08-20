// Topological Data Analysis (TDA) & Persistent Homology (2009 Stanford)
// 논문: "Topology and Data" (Gunnar Carlsson, Bulletin of the AMS 2009)

/// 위상수학적 불변 특징량 (베티 수: Betti Numbers \beta_0, \beta_1)
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalSignature {
    pub betti_0: usize, // 연결 성분 수 (Connected components)
    pub betti_1: usize, // 1차원 위상 구멍 수 (1-D Topological loops / tunnels)
    pub persistence_entropy: f64,
}

/// RAG 벡터 매니폴드 위상 분석기 (TDA Persistent Homology Filter)
pub struct TopologicalDataAnalyzer;

impl TopologicalDataAnalyzer {
    /// 고차원 128차원 벡터 클라우드에서 위상적 불변성(Persistent Invariants) 추출
    pub fn compute_signature(vectors: &[Vec<f32>], epsilon: f32) -> TopologicalSignature {
        if vectors.is_empty() {
            return TopologicalSignature { betti_0: 0, betti_1: 0, persistence_entropy: 0.0 };
        }

        let n = vectors.len();
        let mut adj_matrix = vec![vec![false; n]; n];

        // 1. Vietoris-Rips Complex (\epsilon 근접 이웃 그래프 구성)
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = Self::euclidean_dist(&vectors[i], &vectors[j]);
                if dist <= epsilon {
                    adj_matrix[i][j] = true;
                    adj_matrix[j][i] = true;
                }
            }
        }

        // 2. 연결 성분 계산 (\beta_0: Betti-0)
        let mut visited = vec![false; n];
        let mut betti_0 = 0;
        for i in 0..n {
            if !visited[i] {
                betti_0 += 1;
                Self::dfs(i, &adj_matrix, &mut visited);
            }
        }

        // 3. 단순 삼각 사이클 기반 1차원 구멍 근사 (\beta_1: Betti-1)
        let mut edges_count = 0;
        for i in 0..n {
            for j in (i + 1)..n {
                if adj_matrix[i][j] { edges_count += 1; }
            }
        }
        let betti_1 = if edges_count >= n { edges_count - n + betti_0 } else { 0 };

        // 4. 지속성 엔트로피 (Persistence Entropy)
        let total_features = (betti_0 + betti_1) as f64 + 1e-6;
        let p0 = (betti_0 as f64) / total_features;
        let p1 = (betti_1 as f64) / total_features;
        let entropy = -(p0 * (p0 + 1e-8).ln() + p1 * (p1 + 1e-8).ln());

        TopologicalSignature {
            betti_0,
            betti_1,
            persistence_entropy: entropy,
        }
    }

    fn euclidean_dist(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len().min(b.len());
        a[..len].iter().zip(b[..len].iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }

    fn dfs(node: usize, adj: &[Vec<bool>], visited: &mut [bool]) {
        visited[node] = true;
        for neighbor in 0..adj.len() {
            if adj[node][neighbor] && !visited[neighbor] {
                Self::dfs(neighbor, adj, visited);
            }
        }
    }
}
