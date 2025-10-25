//! Quarantine manager implementing copy-on-write, integrity-verified
//! workflows. All operations are opt-in and reversible until a user
//! explicitly purges artefacts.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::SystemRandom;
use ring::rand::SecureRandom;
use sha2::{Digest, Sha256};

const QUARANTINE_ROOT: &str = "/var/lib/av/quarantine";

#[derive(Debug, Clone)]
pub struct QuarantineConfig {
    pub root: PathBuf,
    pub encryption_key: [u8; 32],
}

impl Default for QuarantineConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from(QUARANTINE_ROOT),
            encryption_key: [0; 32],
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuarantineRecord {
    pub id: String,
    pub original_path: PathBuf,
    pub sha256: String,
    pub size: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct QuarantineManager {
    cfg: QuarantineConfig,
}

impl QuarantineManager {
    pub fn new(cfg: QuarantineConfig) -> anyhow::Result<Self> {
        fs::create_dir_all(&cfg.root)?;
        Ok(Self { cfg })
    }

    /// Copy a file into quarantine using copy-on-write semantics. The source
    /// is left untouched; the caller can remove it only after verifying the
    /// stored artefact.
    pub fn quarantine(&self, path: &Path) -> anyhow::Result<QuarantineRecord> {
        let mut src = File::open(path)?;
        let mut data = Vec::new();
        src.read_to_end(&mut data)?;

        let sha256 = to_hex_hash(&data);
        let size = data.len() as u64;
        let id = format!("{}-{}", chrono::Utc::now().timestamp(), sha256);
        let dest_path = self.cfg.root.join(&id);

        let encrypted = self.encrypt(&data)?;
        let mut dest = OpenOptions::new().create_new(true).write(true).open(&dest_path)?;
        dest.write_all(&encrypted)?;
        dest.flush()?;

        let record = QuarantineRecord {
            id,
            original_path: path.to_path_buf(),
            sha256,
            size,
            timestamp: chrono::Utc::now(),
        };
        self.persist_metadata(&record)?;
        Ok(record)
    }

    pub fn restore(&self, record: &QuarantineRecord, destination: &Path) -> anyhow::Result<()> {
        let encrypted_path = self.cfg.root.join(&record.id);
        let mut encrypted = Vec::new();
        File::open(&encrypted_path)?.read_to_end(&mut encrypted)?;
        let decrypted = self.decrypt(&encrypted)?;

        anyhow::ensure!(to_hex_hash(&decrypted) == record.sha256, "integrity mismatch");

        let mut dest = File::create(destination)?;
        dest.write_all(&decrypted)?;
        Ok(())
    }

    fn persist_metadata(&self, record: &QuarantineRecord) -> anyhow::Result<()> {
        let metadata_path = self.cfg.root.join(format!("{}.json", record.id));
        let mut file = File::create(metadata_path)?;
        let json = serde_json::to_vec_pretty(record)?;
        file.write_all(&json)?;
        Ok(())
    }

    fn encrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, &self.cfg.encryption_key)?);
        let nonce = random_nonce()?;
        let mut buffer = data.to_vec();
        key.seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut buffer)?;
        let mut output = nonce.to_vec();
        output.extend_from_slice(&buffer);
        Ok(output)
    }

    fn decrypt(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(data.len() > 12, "ciphertext too short");
        let (nonce_bytes, cipher) = data.split_at(12);
        let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, &self.cfg.encryption_key)?);
        let mut buffer = cipher.to_vec();
        let decrypted = key.open_in_place(Nonce::assume_unique_for_key(<[u8; 12]>::try_from(nonce_bytes)?), Aad::empty(), &mut buffer)?;
        // ring returns a slice to the decrypted data with the tag removed
        Ok(decrypted.to_vec())
    }
}

fn to_hex_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn random_nonce() -> anyhow::Result<[u8; 12]> {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce)?;
    Ok(nonce)
}
