// BioPhys Core - Complete Interdisciplinary Scientific Engines (2006-2026)
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

pub mod disruptor;
pub mod crdt;
pub mod cuckoo;
pub mod dpll;
pub mod mmr;
pub mod thermo;
pub mod tda;
pub mod fep;
pub mod pqc;
pub mod raft;
pub mod learner;
pub mod compression;
pub mod game_compressor;
pub mod neural_compression;
pub mod vfs_optimizer;
pub mod hyperbolic_engine;
pub mod block_cloner;
pub mod neural_lossless_codec;
pub mod universal_compression_orchestrator;
pub mod unity_asset_optimizer;

pub use disruptor::DisruptorRingBuffer;
pub use crdt::{LwwElementSet, VectorClock};
pub use cuckoo::CuckooFilter;
pub use dpll::{DigitalPhaseLockedLoop, KalmanJitterFilter};
pub use mmr::MerkleMountainRange;
pub use thermo::LandauerReversibleTracker;
pub use tda::{TopologicalDataAnalyzer, TopologicalSignature};
pub use fep::FreeEnergyController;
pub use pqc::{PostQuantumLatticeKex, LatticePolynomial};
pub use raft::{RaftSwarmNode, SwarmRole};
pub use learner::{AutonomousLearner, TargetDomain, SelfLearningReport, StandardKoreanDictionaryEngine};
pub use compression::{ZstdRecordCompressor, VectorBitPacker, SparseSpikeRleCompressor, UnifiedStorageCompression};
pub use game_compressor::{GameStorageOptimizer, GameDirectoryInfo, GameCompressionReport};
pub use neural_compression::{NeuralPredictiveCodec, NeuralCompressionResult};
pub use vfs_optimizer::{GameVfsOptimizer, VfsManifest, VfsFileEntry, VfsChunkMeta};
pub use hyperbolic_engine::{PoincarePoint, CellularAutomataSeed, EpigeneticChromatinGater};
pub use block_cloner::{BlockCloningEngine, BlockCloningReport};
pub use neural_lossless_codec::RansCodec;
pub use universal_compression_orchestrator::{UniversalCompressionOrchestrator, UnifiedRoadmapExecutionReport};
pub use unity_asset_optimizer::{UnityAssetBundleOptimizer, SpineKeyframeCompressor, UnityOptimizationReport};

pub mod neural_compress;

pub mod fractal_vfs;

pub mod bio_fractal_engine;

pub mod bp_arena;

pub mod bp_thread;

pub mod model_runner;

pub mod bpsn_kernel;

pub mod fep_entropy;

pub mod mera_tree;
