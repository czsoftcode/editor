use candle_core::{Device, Tensor};
use candle_transformers::models::bert::{BertModel, Config};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;

/// A single piece of indexed code with its vector embedding.
#[derive(Serialize, Deserialize, Clone)]
pub struct SemanticSnippet {
    pub path: PathBuf,
    pub line_start: usize,
    pub content: String,
    pub embedding: Vec<f32>,
    /// Last modification time of the file when indexed
    pub mtime: u64,
}

/// Local semantic index using BERT embeddings.
pub(crate) struct SemanticIndex {
    pub root: PathBuf,
    pub model: Option<BertModel>,
    pub tokenizer: Option<Tokenizer>,
    pub snippets: Arc<Mutex<Vec<SemanticSnippet>>>,
    pub device: Device,
    pub is_indexing: Arc<AtomicBool>,
    pub files_total: Arc<AtomicUsize>,
    pub files_processed: Arc<AtomicUsize>,
    pub current_file: Arc<Mutex<String>>,
    pub error: Arc<Mutex<Option<String>>>,
}

impl SemanticIndex {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            model: None,
            tokenizer: None,
            snippets: Arc::new(Mutex::new(Vec::new())),
            device: Device::Cpu,
            is_indexing: Arc::new(AtomicBool::new(false)),
            files_total: Arc::new(AtomicUsize::new(0)),
            files_processed: Arc::new(AtomicUsize::new(0)),
            current_file: Arc::new(Mutex::new(String::new())),
            error: Arc::new(Mutex::new(None)),
        }
    }

    pub fn cache_path(&self) -> PathBuf {
        self.root.join(".polycredo").join("semantic_index.bin")
    }

    /// Loads the index from disk if it exists.
    pub fn load(&self) -> anyhow::Result<()> {
        let path = self.cache_path();
        if !path.exists() {
            return Ok(());
        }

        let data = std::fs::read(path)?;
        let loaded_snippets: Vec<SemanticSnippet> = bincode::deserialize(&data)?;

        let mut snippets = self.snippets.lock().unwrap();
        *snippets = loaded_snippets;

        Ok(())
    }

    /// Saves the current index to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let snippets = self.snippets.lock().unwrap();
        let data = bincode::serialize(&*snippets)?;

        let path = self.cache_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, data)?;

        Ok(())
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        use hf_hub::{Repo, api::sync::Api};

        let api = Api::new()?;
        let repo = api.repo(Repo::model(
            "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        ));

        let config_filename = repo.get("config.json")?;
        let tokenizer_filename = repo.get("tokenizer.json")?;
        let weights_filename = repo.get("model.safetensors")?;

        let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(|e| anyhow::anyhow!(e))?;

        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(
                &[weights_filename],
                candle_core::DType::F32,
                &self.device,
            )?
        };
        let model = BertModel::load(vb, &config)?;

        self.model = Some(model);
        self.tokenizer = Some(tokenizer);

        Ok(())
    }

    pub fn vectorize_with_model(
        &self,
        text: &str,
        model: &BertModel,
        tokenizer: &Tokenizer,
    ) -> anyhow::Result<Vec<f32>> {
        let tokens = tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!(e))?;
        let token_ids = tokens.get_ids();
        let input_ids = Tensor::new(token_ids, &self.device)?.unsqueeze(0)?;
        let token_type_ids = input_ids.zeros_like()?;
        let attention_mask = input_ids.ones_like()?;

        let ys = model.forward(&input_ids, &token_type_ids, Some(&attention_mask))?;

        let (_n_batch, n_tokens, _hidden_size) = ys.dims3()?;
        let embeddings = (ys.sum(1)? / (n_tokens as f64))?;
        let result = embeddings.get(0)?.to_vec1::<f32>()?;

        Ok(result)
    }

    pub fn search(
        &self,
        query: &str,
        top_k: usize,
    ) -> anyhow::Result<Vec<(f32, PathBuf, usize, String)>> {
        let Some(model) = &self.model else {
            anyhow::bail!("Model not initialized")
        };
        let Some(tokenizer) = &self.tokenizer else {
            anyhow::bail!("Tokenizer not initialized")
        };

        let query_vec = self.vectorize_with_model(query, model, tokenizer)?;
        let snippets = self.snippets.lock().unwrap();
        let mut results = Vec::new();

        for s in snippets.iter() {
            let similarity = self.cosine_similarity(&query_vec, &s.embedding);
            if similarity > 0.3 {
                results.push((similarity, s.path.clone(), s.line_start, s.content.clone()));
            }
        }

        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        Ok(results)
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot_product / (norm_a * norm_b)
    }
}
