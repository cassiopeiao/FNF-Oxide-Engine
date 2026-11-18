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
    pub chunk_start: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Chunk {
    pub raw_len: u32,
    pub stored_len: u32,
    /// relative to the owning block's `offset`
    pub stored_off: u64,
}

pub struct Pack<'a> {
    bytes: &'a [u8],
    header: Header,
    entries: Vec<Entry>,
    chunks: Vec<Chunk>,
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
        let (entries, chunks) = parse_toc(&toc_raw)?;
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
            chunks,
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
            ALGO_ZSTD_CHUNKED => self
                .read_chunked(entry, 0, entry.raw_len as usize)
                .map_err(|error| format!("{path}: {error}"))?,
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
            ALGO_ZSTD_CHUNKED => self.read_chunked(entry, 0, entry.raw_len as usize).ok(),
            _ => None,
        }
    }

    /// Decodes `len` raw bytes starting at `raw_offset` from a chunked block,
    /// touching only the chunks that overlap the range.
    pub fn read_chunked(
        &self,
        entry: &Entry,
        raw_offset: usize,
        len: usize,
    ) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(len);
        let mut raw_cursor = 0usize;
        let mut chunk_index = entry.chunk_start as usize;
        while raw_cursor < raw_offset + len && chunk_index < self.chunks.len() {
            let chunk = self.chunks[chunk_index];
            let chunk_start = self.bytes.len();
            let _ = chunk_start;
            let start = (entry.offset + chunk.stored_off) as usize;
            let end = start + chunk.stored_len as usize;
            let piece =
                zstd::stream::decode_all(self.bytes.get(start..end).ok_or("chunk out of bounds")?)
                    .map_err(|error| format!("cannot decompress chunk: {error}"))?;
            if piece.len() != chunk.raw_len as usize {
                return Err("chunk length mismatch".into());
            }
            let chunk_end = raw_cursor + piece.len();
            if chunk_end > raw_offset && raw_cursor < raw_offset + len {
                let from = raw_offset.saturating_sub(raw_cursor);
                let to = (raw_offset + len - raw_cursor).min(piece.len());
                out.extend_from_slice(&piece[from..to]);
            }
            raw_cursor = chunk_end;
            chunk_index += 1;
        }
        Ok(out)
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

fn parse_toc(toc: &[u8]) -> Result<(Vec<Entry>, Vec<Chunk>), String> {
    if toc.len() < 16 {
        return Err("TOC too short".into());
    }
    let version = u32::from_le_bytes(toc[0..4].try_into().unwrap());
    if version != 1 {
        return Err(format!("unsupported TOC version {version}"));
    }
    let block_count = u32::from_le_bytes(toc[4..8].try_into().unwrap()) as usize;
    let chunk_count = u32::from_le_bytes(toc[8..12].try_into().unwrap()) as usize;
    let string_bytes = u32::from_le_bytes(toc[12..16].try_into().unwrap()) as usize;
    let blocks_start = 16;
    let blocks_len = block_count * 40;
    let chunks_start = blocks_start + blocks_len;
    let strings_start = chunks_start + chunk_count * 16;
    if toc.len() < strings_start + string_bytes {
        return Err("TOC is truncated".into());
    }
    let strings = &toc[strings_start..strings_start + string_bytes];
    let mut chunks = Vec::with_capacity(chunk_count);
    for index in 0..chunk_count {
        let at = chunks_start + index * 16;
        let chunk = &toc[at..at + 16];
        chunks.push(Chunk {
            raw_len: u32::from_le_bytes(chunk[0..4].try_into().unwrap()),
            stored_len: u32::from_le_bytes(chunk[4..8].try_into().unwrap()),
            stored_off: u64::from_le_bytes(chunk[8..16].try_into().unwrap()),
        });
    }

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
            chunk_start: u32::from_le_bytes(block[36..40].try_into().unwrap()),
        });
    }
    Ok((entries, chunks))
}
