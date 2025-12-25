use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use std::io::Cursor;
// Import the CPU-based Embedding Engine from inference.rs
use crate::inference::EmbeddingEngine;

/// THE HIPPOCAMPUS (v2.1)
/// A Biomimetic, Compressed, Multimodal Memory System.
/// 
/// Core Features:
/// 1. Micro-Memory (Chunks) for granular retrieval.
/// 2. Spreading Activation (Synapses) for associative recall.
/// 3. Long-Term Potentiation (LTP) for learning user habits.
/// 4. Z-Layer (zstd) for storage optimization (<5% overhead).

// --- DATA STRUCTURES ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Code,   // Indentation-based chunking
    Prose,  // Paragraph-based chunking
    Config, // Line-based chunking
    Unknown,
}

/// The atomic unit of retrieval.
/// Represents a specific segment (paragraph/function) within a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: usize,           // 0, 1, 2...
    pub start_byte: usize,
    pub end_byte: usize,
    pub embedding: Vec<f32>, // 384-dim Vector (all-MiniLM-L6-v2)
    
    // BIOMIMETIC: Long-Term Potentiation
    // Increments on successful recall. Multiplier = 1.0 + (count * 0.01)
    #[serde(default)]
    pub access_count: u32,
}

/// The "Synapse" - A weighted connection between files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    pub target: PathBuf,
    pub strength: f32, // 0.2 (Weak) to 5.0 (Forced)
}

/// The Deep Store (Disk Resident)
/// Contains the heavy data, compressed to save space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engram {
    pub path: PathBuf,
    pub file_type: FileType,
    pub hash: String, // SHA256 for differential indexing
    
    // COMPRESSED DATA (Z-Layer)
    // We store these as raw bytes to avoid string overhead before decompression
    pub skeleton_compressed: Vec<u8>, 
    
    // Inverted Index: "Topic" -> [ChunkID, ChunkID]
    pub entity_map: HashMap<String, Vec<usize>>,
    
    // Graph Edges
    pub synapses: Vec<Synapse>,
    
    // The Chunks
    pub chunks: Vec<Chunk>,
}

// --- MEMORY SYSTEM ---

pub struct MemorySystem {
    // Disk Cache (The Bookshelf)
    library: HashMap<PathBuf, Engram>,
    
    // Fast RAM Index (The Card Catalog)
    // Optimized for O(N) linear scanning
    flat_index: Vec<(PathBuf, usize, Vec<f32>)>, 
}

impl MemorySystem {
    pub fn new() -> Self {
        Self {
            library: HashMap::new(),
            flat_index: Vec::new(),
        }
    }

    // --- PIPELINE A: INGESTION (The Lazy Indexer) ---

    pub fn index_file(&mut self, path: PathBuf, embedder: &EmbeddingEngine) -> Result<(), String> {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let file_type = self.detect_type(&path);
        
        // 1. Chunking
        let text_chunks = self.chunk_content(&content, &file_type);
        
        // 2. Vector Embedding (Real CPU Inference)
        let mut chunks = Vec::new();
        for (i, (start, end, text_slice)) in text_chunks.into_iter().enumerate() {
            // Generate real vector using the passed EmbeddingEngine
            let embedding = embedder.embed(&text_slice);
            
            chunks.push(Chunk {
                id: i,
                start_byte: start,
                end_byte: end,
                embedding: embedding.clone(),
                access_count: 0,
            });
            
            // Add to Fast Index for O(N) scanning
            self.flat_index.push((path.clone(), i, embedding));
        }
        
        // 3. Skeletonization & Compression (Z-Layer)
        // Stub: In production, we'd use the LLM to generate a summary first
        let skeleton_raw = format!("Skeleton for {:?}", path); 
        // Compress using Level 3 (Balanced)
        let skeleton_compressed = zstd::encode_all(Cursor::new(skeleton_raw), 3)
            .map_err(|e| format!("Compression failed: {}", e))?;

        let engram = Engram {
            path: path.clone(),
            file_type,
            hash: "SHA256_STUB".to_string(), // TODO: Implement hashing
            skeleton_compressed,
            entity_map: HashMap::new(), // TODO: Sidekick Entity Extraction
            synapses: Vec::new(),       // TODO: Regex Link Extraction
            chunks,
        };

        self.library.insert(path, engram);
        Ok(())
    }

    // --- PIPELINE B: RETRIEVAL (Biomimetic) ---

    pub fn retrieve_context(&mut self, query_text: &str, query_vec: &[f32]) -> Vec<String> {
        // Map of (Path, ChunkID) -> Activation Score
        let mut activations: HashMap<(PathBuf, usize), f32> = HashMap::new();

        // PHASE 1: INITIAL ACTIVATION
        
        // A. Entity Hits (Keyword Match)
        for (path, engram) in &self.library {
            for (entity, ids) in &engram.entity_map {
                if query_text.contains(entity) {
                    for id in ids {
                        // High base score for exact keyword matches
                        activations.insert((path.clone(), *id), 1.5);
                    }
                }
            }
        }

        // B. Vector Hits (Semantic Match) + LTP
        for (path, chunk_idx, emb) in &self.flat_index {
            let similarity = cosine_similarity(query_vec, emb);
            
            // LTP BOOST: Frequent memories are stronger
            // Memories accessed 100 times get a 2x multiplier
            let mut boost = 1.0;
            if let Some(engram) = self.library.get(path) {
                if let Some(chunk) = engram.chunks.get(*chunk_idx) {
                    boost += chunk.access_count as f32 * 0.01;
                }
            }
            
            let score = similarity * boost;
            if score > 0.7 {
                 let entry = activations.entry((path.clone(), *chunk_idx)).or_insert(0.0);
                 *entry = entry.max(score);
            }
        }

        // PHASE 2: SPREADING ACTIVATION (The Synaptic Hop)
        let mut synaptic_boosts: Vec<((PathBuf, usize), f32)> = Vec::new();

        for ((path, _), score) in &activations {
            if *score > 0.8 { // Only strong signals propagate
                if let Some(engram) = self.library.get(path) {
                    for synapse in &engram.synapses {
                        // Formula: Origin * Strength * Decay
                        let transmission = score * synapse.strength * 0.5;
                        
                        // Activate Chunk 0 (Summary) of target file
                        synaptic_boosts.push(((synapse.target.clone(), 0), transmission));
                    }
                }
            }
        }

        // Apply synaptic boosts
        for (key, boost) in synaptic_boosts {
            let entry = activations.entry(key).or_insert(0.0);
            *entry += boost;
        }

        // PHASE 3: ARCHIPELAGO ASSEMBLY (Merging)
        // 1. Filter Top Hits
        let top_hits: HashSet<(PathBuf, usize)> = activations.into_iter()
            .filter(|(_, score)| *score > 0.75)
            .map(|(key, _)| key)
            .collect();
            
        // 2. Group by File
        let mut file_hits: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        for (path, idx) in top_hits {
            file_hits.entry(path).or_default().push(idx);
        }

        // 3. Expand & Read (The Context Window)
        let mut context_blocks = Vec::new();
        for (path, hits) in file_hits {
             // Reinforce these memories (LTP) so they are easier to find next time
             for &h in &hits { self.reinforce(&path, h); }
             
             // In a real implementation, we would merge adjacent chunks (Islands) here.
             // For now, we grab the raw chunks.
             if let Ok(content) = fs::read_to_string(&path) {
                 if let Some(engram) = self.library.get(&path) {
                     for idx in hits {
                         if let Some(chunk) = engram.chunks.get(idx) {
                             if chunk.end_byte <= content.len() {
                                 context_blocks.push(content[chunk.start_byte..chunk.end_byte].to_string());
                             }
                         }
                     }
                 }
             }
        }

        context_blocks
    }

    // --- UTILS ---

    /// Manually strengthen a memory path (LTP)
    pub fn reinforce(&mut self, path: &PathBuf, chunk_id: usize) {
        if let Some(engram) = self.library.get_mut(path) {
            if let Some(chunk) = engram.chunks.get_mut(chunk_id) {
                chunk.access_count += 1;
            }
        }
    }

    /// create a manual link between two files
    pub fn forge_synapse(&mut self, source: PathBuf, target: PathBuf, strength: f32) {
        if let Some(engram) = self.library.get_mut(&source) {
            engram.synapses.push(Synapse { target, strength });
        }
    }

    fn detect_type(&self, path: &Path) -> FileType {
         match path.extension().and_then(|s| s.to_str()) {
            Some("rs") | Some("py") | Some("c") | Some("cpp") => FileType::Code,
            Some("md") | Some("txt") => FileType::Prose,
            Some("json") | Some("toml") | Some("yaml") => FileType::Config,
            _ => FileType::Unknown,
        }
    }

    fn chunk_content(&self, content: &str, ftype: &FileType) -> Vec<(usize, usize, String)> {
        let mut chunks = Vec::new();
        let mut start = 0;
        // Basic heuristic splitting
        let pattern = match ftype {
            FileType::Prose => "\n\n", // Paragraphs
            _ => "\n}", // Code Blocks (Crude heuristic)
        };
        
        for part in content.split(pattern) {
            let len = part.len();
            let end = start + len;
            // Filter out noise/whitespace
            if len > 20 {
                chunks.push((start, end, part.to_string()));
            }
            start = end + pattern.len();
        }
        chunks
    }
}

// Simple Cosine Similarity Helper
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}