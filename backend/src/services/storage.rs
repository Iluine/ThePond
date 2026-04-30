//! Layout disque des uploads (PROJECT.md § "Stockage des fichiers").
//!
//! ```
//! {uploads_path}/
//! ├── photos/
//! │   ├── original/{uuid}.{ext}
//! │   └── thumb/{uuid}.jpg
//! ├── clips/
//! │   ├── original/{uuid}.{ext}
//! │   └── thumb/{uuid}.jpg
//! └── voice/
//!     └── {uuid}.{ext}
//! ```

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone)]
pub struct Storage {
    root: PathBuf,
}

impl Storage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Crée tous les sous-dossiers nécessaires si absents (idempotent),
    /// puis génère le placeholder.jpg de clip si absent.
    pub fn ensure_dirs(&self) -> Result<()> {
        for sub in [
            "photos/original",
            "photos/thumb",
            "clips/original",
            "clips/thumb",
            "voice",
        ] {
            let p = self.root.join(sub);
            std::fs::create_dir_all(&p)
                .with_context(|| format!("mkdir -p {}", p.display()))?;
        }
        self.ensure_clip_placeholder()?;
        Ok(())
    }

    /// Le clip placeholder est une image 600×600 cream uniforme. Tous les
    /// clips le réutilisent comme thumbnail tant que la pipeline ffmpeg
    /// d'extraction de frame n'est pas livrée. Une fois la pipeline
    /// branchée, le job replacera ce nom par un thumb par-clip.
    pub const CLIP_PLACEHOLDER_FILENAME: &'static str = "clips/thumb/_placeholder.jpg";

    fn ensure_clip_placeholder(&self) -> Result<()> {
        let p = self.root.join(Self::CLIP_PLACEHOLDER_FILENAME);
        if p.exists() {
            return Ok(());
        }
        // Cream uniforme #FAF3E3
        let img = image::RgbImage::from_pixel(600, 600, image::Rgb([0xFA, 0xF3, 0xE3]));
        img.save(&p)
            .with_context(|| format!("write clip placeholder to {}", p.display()))?;
        Ok(())
    }

    /// Retourne (filename relatif, full path absolu) pour un nouveau
    /// fichier dans la sous-catégorie donnée. filename est ce qu'on
    /// stocke en DB, full path est ce qu'on écrit sur disque.
    pub fn allocate(&self, kind: Kind, ext: &str) -> (String, PathBuf) {
        let id = Uuid::new_v4();
        let filename = format!("{id}.{ext}");
        let path = self.root.join(kind.subdir()).join(&filename);
        (format!("{}/{}", kind.subdir(), filename), path)
    }

    /// Retourne le filename stocké en DB pour le thumbnail correspondant
    /// à un fichier original. Pour photos/clips on dérive le nom
    /// (mêmes uuid, extension .jpg, sous-dossier thumb).
    pub fn thumb_for(&self, kind: Kind, original_filename: &str) -> (String, PathBuf) {
        let stem = Path::new(original_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let thumb_name = format!("{stem}.jpg");
        let thumb_subdir = kind.thumb_subdir();
        let path = self.root.join(thumb_subdir).join(&thumb_name);
        (format!("{thumb_subdir}/{thumb_name}"), path)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    PhotoOriginal,
    ClipOriginal,
    Voice,
}

impl Kind {
    fn subdir(self) -> &'static str {
        match self {
            Self::PhotoOriginal => "photos/original",
            Self::ClipOriginal => "clips/original",
            Self::Voice => "voice",
        }
    }

    fn thumb_subdir(self) -> &'static str {
        match self {
            Self::PhotoOriginal => "photos/thumb",
            Self::ClipOriginal => "clips/thumb",
            Self::Voice => "voice", // pas utilisé — voice n'a pas de thumb
        }
    }
}
