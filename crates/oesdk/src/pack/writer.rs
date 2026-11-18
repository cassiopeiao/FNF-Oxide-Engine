//! Streaming-friendly writer. Takes all blocks in memory for now; the layout
//! already allows writing blocks as they arrive.

use super::*;
use crate::pack::merkle::{TOC_PATH, leaf_hash, root, sign_message};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

pub struct Entry {
    pub path: String,
    pub kind: u8,
    pub raw: Vec<u8>,
    pub stored: Vec<u8>,
    pub algo: u8,
    pub offset: u64,
}

#[derive(Default)]
pub struct PackBuilder {
    entries: Vec<Entry>,
    schema_hash: [u8; 32],
    zstd_level: i32,
}

impl PackBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            schema_hash: [0; 32],
            zstd_level: 19,
        }
    }

    pub fn schema_hash(&mut self, hash: [u8; 32]) -> &mut Self {
        self.schema_hash = hash;
        self
    }

    /// Adds a file. The compression is chosen from the path; callers that know
    /// better can use [`Self::add_with_algo`].
    pub fn add(&mut self, path: impl Into<String>, raw: Vec<u8>) -> Result<&mut Self, String> {
        let path = path.into();
        let algo = if is_precompressed(&path) {
            ALGO_STORE
        } else if is_text(&path) {
            ALGO_ZSTD
        } else {
            ALGO_STORE
        };
        let kind = kind_for(&path);
        self.add_with_algo(path, raw, algo, kind)
    }

    pub fn add_with_algo(
        &mut self,
        path: impl Into<String>,
        raw: Vec<u8>,
        algo: u8,
        kind: u8,
    ) -> Result<&mut Self, String> {
        let path = path.into();
        validate_path(&path)?;
        if self.entries.iter().any(|entry| entry.path == path) {
            return Err(format!("duplicate path in package: {path}"));
        }
        let stored = match algo {
            ALGO_STORE => raw.clone(),
            ALGO_ZSTD => zstd::stream::encode_all(raw.as_slice(), self.zstd_level)
                .map_err(|error| format!("zstd failed for {path}: {error}"))?,
            _ => return Err(format!("unsupported algo {algo} for {path}")),
        };
        self.entries.push(Entry {
            path,
            kind,
            raw,
            stored,
            algo,
            offset: 0,
        });
        Ok(self)
    }

    pub fn finish(&mut self, key: Option<&SigningKey>) -> Result<Vec<u8>, String> {
        self.entries
            .sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));

        let mut out = vec![0u8; HEADER_LEN as usize];
        let mut cursor = HEADER_LEN as u64;
        for entry in &mut self.entries {
            cursor = align_up(cursor, ALIGN);
            out.resize(cursor as usize, 0);
            entry.offset = cursor;
            out.extend_from_slice(&entry.stored);
            cursor += entry.stored.len() as u64;
        }

        let toc_raw = self.toc_bytes();
        let toc = zstd::stream::encode_all(toc_raw.as_slice(), 19)
            .map_err(|error| format!("zstd failed for the TOC: {error}"))?;
        let toc_off = align_up(cursor, ALIGN);
        out.resize(toc_off as usize, 0);
        out.extend_from_slice(&toc);

        let mut leaves: Vec<[u8; 32]> = self
            .entries
            .iter()
            .map(|entry| leaf_hash(&entry.path, &entry.raw))
            .collect();
        leaves.push(leaf_hash(TOC_PATH, &toc_raw));
        let merkle_root = root(&leaves);

        let mut header = Header {
            flags: FLAG_TOC_ZSTD,
            toc_off,
            toc_len: toc.len() as u64,
            toc_raw_len: toc_raw.len() as u64,
            block_count: self.entries.len() as u32,
            chunk_count: 0,
            merkle_root,
            schema_hash: self.schema_hash,
            ..Header::new()
        };
        if let Some(key) = key {
            let signature = key.sign(&sign_message(&merkle_root));
            header.signature = signature.to_bytes();
            header.flags |= FLAG_SIGNED;
        }
        let mut head = Vec::with_capacity(HEADER_LEN as usize);
        header.write(&mut head);
        out[..HEADER_LEN as usize].copy_from_slice(&head);
        Ok(out)
    }

    fn toc_bytes(&self) -> Vec<u8> {
        let mut strings = Vec::new();
        let mut blocks = Vec::new();
        for entry in &self.entries {
            let path_off = strings.len() as u32;
            strings.extend_from_slice(entry.path.as_bytes());
            strings.push(0);
            blocks.extend_from_slice(&path_off.to_le_bytes());
            blocks.extend_from_slice(&(entry.path.len() as u16).to_le_bytes());
            blocks.push(entry.kind);
            blocks.push(entry.algo);
            blocks.extend_from_slice(&entry.offset.to_le_bytes());
            blocks.extend_from_slice(&(entry.stored.len() as u64).to_le_bytes());
            blocks.extend_from_slice(&(entry.raw.len() as u64).to_le_bytes());
            blocks.extend_from_slice(&crc32c(&entry.raw).to_le_bytes());
            blocks.extend_from_slice(&0u32.to_le_bytes());
        }

        let mut toc = Vec::new();
        toc.extend_from_slice(&1u32.to_le_bytes());
        toc.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        toc.extend_from_slice(&0u32.to_le_bytes());
        toc.extend_from_slice(&(strings.len() as u32).to_le_bytes());
        toc.extend_from_slice(&blocks);
        toc.extend_from_slice(&strings);
        toc
    }
}

pub fn crc32c(bytes: &[u8]) -> u32 {
    crc32fast::hash(bytes)
}

pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return Err(format!("invalid package path: {path:?}"));
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(format!("invalid package path: {path:?}"));
        }
    }
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::reader::Pack;

    #[test]
    fn roundtrip_and_verify() {
        let key = SigningKey::from_bytes(&[7u8; 32]);
        let mut builder = PackBuilder::new();
        builder.add("mod.wasm", vec![1, 2, 3, 4]).unwrap();
        builder
            .add("content/stage.toml", b"id = \"x\"\n".to_vec())
            .unwrap();
        let bytes = builder.finish(Some(&key)).unwrap();

        let pack = Pack::parse(&bytes).unwrap();
        assert_eq!(pack.entries().len(), 2);
        assert!(pack.verify_signature(&key.verifying_key()).is_ok());
        assert_eq!(pack.read("mod.wasm").unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn detects_tampering() {
        let key = SigningKey::from_bytes(&[9u8; 32]);
        let mut builder = PackBuilder::new();
        builder.add("data.bin", vec![9u8; 64]).unwrap();
        let mut bytes = builder.finish(Some(&key)).unwrap();
        let pack = Pack::parse(&bytes).unwrap();
        let offset = pack.entry("data.bin").unwrap().offset as usize;
        bytes[offset] ^= 0xff;
        let pack = Pack::parse(&bytes).unwrap();
        assert!(pack.read("data.bin").is_err());
    }
}
