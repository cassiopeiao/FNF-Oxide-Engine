//! Verifying reader. Every access checks length, CRC and the Merkle leaf.

use super::*;
use crate::pack::merkle::{TOC_PATH, leaf_hash, root, sign_message};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

#[derive(Clone, Debug)]
pub struct Entry {
    pub path: String,
    pub kind: u8,
    pub offset: u64,
    pub stored_len: u64,
    pub raw_len: u64,
    pub crc32: u32,
    pub algo: u8,
}

pub struct Pack<'a> {
    bytes: &'a [u8],
    header: Header,
    entries: Vec<Entry>,
}

impl<'a> Pack<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, String> {
        let header = Header::parse(bytes)?;
        let toc_end = header.toc_off + header.toc_len;
        if toc_end > bytes.len() as u64 {
            return Err("TOC extends past the end of the file".into());
        }
        let toc_raw = zstd::stream::decode_all(&bytes[header.toc_off as usize..toc_end as usize])
            .map_err(|error| format!("cannot decompress the TOC: {error}"))?;
        if toc_raw.len() as u64 != header.toc_raw_len {
            return Err("TOC length mismatch".into());
        }
        let entries = parse_toc(&toc_raw)?;
        if entries.len() as u32 != header.block_count {
            return Err("block count mismatch".into());
        }
        for entry in &entries {
            if entry.offset + entry.stored_len > header.toc_off {
                return Err(format!("block {} overlaps the TOC", entry.path));
            }
        }
        Ok(Self {
            bytes,
            header,
            entries,
        })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn entry(&self, path: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.path == path)
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn read(&self, path: &str) -> Result<Vec<u8>, String> {
        let entry = self
            .entry(path)
            .ok_or_else(|| format!("no such entry: {path}"))?;
        let start = entry.offset as usize;
        let end = start + entry.stored_len as usize;
        let raw = match entry.algo {
            ALGO_STORE => self.bytes[start..end].to_vec(),
            ALGO_ZSTD => zstd::stream::decode_all(&self.bytes[start..end])
                .map_err(|error| format!("cannot decompress {path}: {error}"))?,
            other => return Err(format!("{path}: unsupported algorithm {other}")),
        };
        if raw.len() as u64 != entry.raw_len {
            return Err(format!("{path}: length mismatch"));
        }
        if crc32fast::hash(&raw) != entry.crc32 {
            return Err(format!("{path}: CRC mismatch"));
        }
        Ok(raw)
    }

    fn read_raw_unchecked(&self, entry: &Entry) -> Option<Vec<u8>> {
        let start = entry.offset as usize;
        let end = start + entry.stored_len as usize;
        match entry.algo {
            ALGO_STORE => self.bytes.get(start..end).map(<[u8]>::to_vec),
            ALGO_ZSTD => zstd::stream::decode_all(self.bytes.get(start..end)?).ok(),
            _ => None,
        }
    }

    fn toc_raw(&self) -> Vec<u8> {
        let start = self.header.toc_off as usize;
        let end = start + self.header.toc_len as usize;
        zstd::stream::decode_all(&self.bytes[start..end]).unwrap_or_default()
    }

    /// Rebuilds the root from the blocks and checks the signature. O(pack).
    pub fn verify_signature(&self, trusted: &VerifyingKey) -> Result<(), String> {
        if self.header.flags & FLAG_SIGNED == 0 {
            return Err("package is not signed".into());
        }
        let mut leaves = Vec::with_capacity(self.entries.len() + 1);
        for entry in &self.entries {
            let raw = self.read_raw_unchecked(entry).ok_or("cannot read block")?;
            leaves.push(leaf_hash(&entry.path, &raw));
        }
        leaves.push(leaf_hash(TOC_PATH, &self.toc_raw()));
        let computed = root(&leaves);
        if computed != self.header.merkle_root {
            return Err("Merkle root does not match the blocks".into());
        }
        let signature = Signature::from_bytes(&self.header.signature);
        trusted
            .verify(&sign_message(&computed), &signature)
            .map_err(|_| "signature does not match the package".to_string())
    }
}

fn parse_toc(toc: &[u8]) -> Result<Vec<Entry>, String> {
    if toc.len() < 16 {
        return Err("TOC too short".into());
    }
    let version = u32::from_le_bytes(toc[0..4].try_into().unwrap());
    if version != 1 {
        return Err(format!("unsupported TOC version {version}"));
    }
    let block_count = u32::from_le_bytes(toc[4..8].try_into().unwrap()) as usize;
    let string_bytes = u32::from_le_bytes(toc[12..16].try_into().unwrap()) as usize;
    let blocks_start = 16;
    let blocks_len = block_count * 40;
    let strings_start = blocks_start + blocks_len;
    if toc.len() < strings_start + string_bytes {
        return Err("TOC is truncated".into());
    }
    let strings = &toc[strings_start..strings_start + string_bytes];

    let mut entries = Vec::with_capacity(block_count);
    for index in 0..block_count {
        let at = blocks_start + index * 40;
        let block = &toc[at..at + 40];
        let path_off = u32::from_le_bytes(block[0..4].try_into().unwrap()) as usize;
        let path_len = u16::from_le_bytes(block[4..6].try_into().unwrap()) as usize;
        let path_end = path_off + path_len;
        if path_end > strings.len() {
            return Err("TOC path out of bounds".into());
        }
        let path = std::str::from_utf8(&strings[path_off..path_end])
            .map_err(|_| "TOC path is not UTF-8".to_string())?
            .to_string();
        crate::pack::writer::validate_path(&path)?;
        entries.push(Entry {
            path,
            kind: block[6],
            offset: u64::from_le_bytes(block[8..16].try_into().unwrap()),
            stored_len: u64::from_le_bytes(block[16..24].try_into().unwrap()),
            raw_len: u64::from_le_bytes(block[24..32].try_into().unwrap()),
            crc32: u32::from_le_bytes(block[32..36].try_into().unwrap()),
            algo: block[7],
        });
    }
    Ok(entries)
}
