//! Whisper model management and downloading

use anyhow::{anyhow, Result};
use dirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Available Whisper model sizes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WhisperModel {
    Tiny,
    TinyEn,
    Base,
    BaseEn,
    Small,
    SmallEn,
    Medium,
    MediumEn,
    Large,
    LargeV2,
    LargeV3,
}

impl WhisperModel {
    /// Get model filename
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Tiny => "ggml-tiny.bin",
            Self::TinyEn => "ggml-tiny.en.bin",
            Self::Base => "ggml-base.bin",
            Self::BaseEn => "ggml-base.en.bin",
            Self::Small => "ggml-small.bin",
            Self::SmallEn => "ggml-small.en.bin",
            Self::Medium => "ggml-medium.bin",
            Self::MediumEn => "ggml-medium.en.bin",
            Self::Large => "ggml-large.bin",
            Self::LargeV2 => "ggml-large-v2.bin",
            Self::LargeV3 => "ggml-large-v3.bin",
        }
    }

    /// Get download URL
    pub fn download_url(&self) -> &'static str {
        match self {
            Self::Tiny => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
            Self::TinyEn => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin",
            Self::Base => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
            Self::BaseEn => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
            Self::Small => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
            Self::SmallEn => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
            Self::Medium => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
            Self::MediumEn => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin",
            Self::Large => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large.bin",
            Self::LargeV2 => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v2.bin",
            Self::LargeV3 => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin",
        }
    }

    /// Get approximate model size in MB
    pub fn size_mb(&self) -> u64 {
        match self {
            Self::Tiny | Self::TinyEn => 39,
            Self::Base | Self::BaseEn => 142,
            Self::Small | Self::SmallEn => 466,
            Self::Medium | Self::MediumEn => 1420,
            Self::Large => 2880,
            Self::LargeV2 => 2880,
            Self::LargeV3 => 2880,
        }
    }

    /// Get model description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Tiny => "Tiny multilingual model (39 MB)",
            Self::TinyEn => "Tiny English-only model (39 MB)",
            Self::Base => "Base multilingual model (142 MB)",
            Self::BaseEn => "Base English-only model (142 MB)",
            Self::Small => "Small multilingual model (466 MB)",
            Self::SmallEn => "Small English-only model (466 MB)",
            Self::Medium => "Medium multilingual model (1.4 GB)",
            Self::MediumEn => "Medium English-only model (1.4 GB)",
            Self::Large => "Large multilingual model (2.9 GB)",
            Self::LargeV2 => "Large v2 multilingual model (2.9 GB)",
            Self::LargeV3 => "Large v3 multilingual model (2.9 GB)",
        }
    }

    /// Check if model is English-only
    pub fn is_english_only(&self) -> bool {
        matches!(self, Self::TinyEn | Self::BaseEn | Self::SmallEn | Self::MediumEn)
    }

    /// Get recommended model for different use cases
    pub fn recommended_for_speed() -> Self {
        Self::BaseEn
    }

    pub fn recommended_for_accuracy() -> Self {
        Self::LargeV3
    }

    pub fn recommended_for_size() -> Self {
        Self::TinyEn
    }
}

impl std::fmt::Display for WhisperModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for WhisperModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tiny" => Ok(Self::Tiny),
            "tiny.en" | "tiny-en" => Ok(Self::TinyEn),
            "base" => Ok(Self::Base),
            "base.en" | "base-en" => Ok(Self::BaseEn),
            "small" => Ok(Self::Small),
            "small.en" | "small-en" => Ok(Self::SmallEn),
            "medium" => Ok(Self::Medium),
            "medium.en" | "medium-en" => Ok(Self::MediumEn),
            "large" => Ok(Self::Large),
            "large-v2" | "large_v2" => Ok(Self::LargeV2),
            "large-v3" | "large_v3" => Ok(Self::LargeV3),
            _ => Err(anyhow!("Unknown model: {}", s)),
        }
    }
}

/// Model manager for downloading and managing Whisper models
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    /// Create new model manager
    pub fn new() -> Result<Self> {
        let models_dir = Self::get_models_dir()?;
        std::fs::create_dir_all(&models_dir)?;
        
        Ok(Self { models_dir })
    }

    /// Get models directory
    fn get_models_dir() -> Result<PathBuf> {
        if let Some(data_dir) = dirs::data_dir() {
            Ok(data_dir.join("savant-ai").join("models"))
        } else {
            Ok(PathBuf::from("./models"))
        }
    }

    /// Get path to model file
    pub fn get_model_path(&self, model: &WhisperModel) -> PathBuf {
        self.models_dir.join(model.filename())
    }

    /// Check if model is downloaded
    pub fn is_model_available(&self, model: &WhisperModel) -> bool {
        let path = self.get_model_path(model);
        path.exists() && path.is_file()
    }

    /// List available models
    pub fn list_available_models(&self) -> Vec<WhisperModel> {
        let models = [
            WhisperModel::Tiny,
            WhisperModel::TinyEn,
            WhisperModel::Base,
            WhisperModel::BaseEn,
            WhisperModel::Small,
            WhisperModel::SmallEn,
            WhisperModel::Medium,
            WhisperModel::MediumEn,
            WhisperModel::Large,
            WhisperModel::LargeV2,
            WhisperModel::LargeV3,
        ];

        models
            .iter()
            .filter(|model| self.is_model_available(model))
            .cloned()
            .collect()
    }

    /// Download model
    pub async fn download_model(&self, model: &WhisperModel) -> Result<PathBuf> {
        let model_path = self.get_model_path(model);
        
        if model_path.exists() {
            info!("Model {} already exists", model.filename());
            return Ok(model_path);
        }

        info!("Downloading model {} ({} MB)...", model.filename(), model.size_mb());

        let url = model.download_url();
        let response = reqwest::get(url).await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download model: HTTP {}", response.status()));
        }

        let content = response.bytes().await?;
        std::fs::write(&model_path, content)?;

        info!("Successfully downloaded model to {}", model_path.display());
        Ok(model_path)
    }

    /// Get best available model for current system
    pub fn get_best_available_model(&self) -> Option<WhisperModel> {
        let available = self.list_available_models();
        
        if available.is_empty() {
            return None;
        }

        // Prefer larger models for better accuracy
        for model in [
            WhisperModel::LargeV3,
            WhisperModel::LargeV2,
            WhisperModel::Large,
            WhisperModel::Medium,
            WhisperModel::MediumEn,
            WhisperModel::Small,
            WhisperModel::SmallEn,
            WhisperModel::Base,
            WhisperModel::BaseEn,
            WhisperModel::Tiny,
            WhisperModel::TinyEn,
        ] {
            if available.contains(&model) {
                return Some(model);
            }
        }

        None
    }

    /// Ensure a model is available, downloading if necessary
    pub async fn ensure_model(&self, model: &WhisperModel) -> Result<PathBuf> {
        if self.is_model_available(model) {
            Ok(self.get_model_path(model))
        } else {
            self.download_model(model).await
        }
    }

    /// Delete model
    pub fn delete_model(&self, model: &WhisperModel) -> Result<()> {
        let model_path = self.get_model_path(model);
        
        if model_path.exists() {
            std::fs::remove_file(&model_path)?;
            info!("Deleted model {}", model.filename());
        } else {
            warn!("Model {} not found", model.filename());
        }

        Ok(())
    }

    /// Get models directory path
    pub fn models_directory(&self) -> &Path {
        &self.models_dir
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new().expect("Failed to create model manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_properties() {
        let model = WhisperModel::BaseEn;
        assert_eq!(model.filename(), "ggml-base.en.bin");
        assert!(model.is_english_only());
        assert_eq!(model.size_mb(), 142);
    }

    #[test]
    fn test_model_parsing() {
        assert_eq!("base.en".parse::<WhisperModel>().unwrap(), WhisperModel::BaseEn);
        assert_eq!("large-v3".parse::<WhisperModel>().unwrap(), WhisperModel::LargeV3);
        assert!("invalid".parse::<WhisperModel>().is_err());
    }

    #[tokio::test]
    async fn test_model_manager() {
        let manager = ModelManager::new().unwrap();
        
        // Test model path generation
        let path = manager.get_model_path(&WhisperModel::BaseEn);
        assert!(path.to_string_lossy().contains("ggml-base.en.bin"));
        
        // Test model availability check
        let available = manager.is_model_available(&WhisperModel::BaseEn);
        // Should be false unless model is actually downloaded
        assert!(!available || path.exists());
    }
}