//! `.fnfm` package format v1 — see `docs/modpack-v1.md`.
//!
//! Header layout (176 bytes, little-endian):
//! ```text
//!  0..4   magic "FNFM"      44..76   merkle_root
//!  4..6   format_ver        76..108  schema_hash
//!  6..8   header_len       108..172  signature
//!  8..12  flags            172..176  reserved (0)
//! 12..20  toc_off
//! 20..28  toc_len
//! 28..36  toc_raw_len
//! 36..40  block_count
//! 40..44  chunk_count
//! ```

pub mod merkle;
pub mod reader;
pub mod writer;

pub const MAGIC: [u8; 4] = *b"FNFM";
pub const HEADER_LEN: u32 = 176;
/// major in the high byte, minor in the low byte
pub const VERSION: u16 = 0x0100;

pub const FLAG_SIGNED: u32 = 1 << 0;
pub const FLAG_TOC_ZSTD: u32 = 1 << 1;

pub const KIND_PROTO: u8 = 0;
pub const KIND_IMAGE_RGBA: u8 = 1;
pub const KIND_AUDIO: u8 = 2;
pub const KIND_RAW: u8 = 3;
pub const KIND_MANIFEST: u8 = 4;

pub const ALGO_STORE: u8 = 0;
pub const ALGO_ZSTD: u8 = 1;
pub const ALGO_ZSTD_CHUNKED: u8 = 2;

/// Every block starts on a 16-byte boundary.
pub const ALIGN: u64 = 16;

const OFF_MERKLE: usize = 44;
const OFF_SCHEMA: usize = 76;
const OFF_SIG: usize = 108;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub format_ver: u16,
    pub flags: u32,
    pub toc_off: u64,
    pub toc_len: u64,
    pub toc_raw_len: u64,
    pub block_count: u32,
    pub chunk_count: u32,
    pub merkle_root: [u8; 32],
    pub schema_hash: [u8; 32],
    pub signature: [u8; 64],
}

impl Header {
    pub fn new() -> Self {
        Self {
            format_ver: VERSION,
            flags: 0,
            toc_off: 0,
            toc_len: 0,
            toc_raw_len: 0,
            block_count: 0,
            chunk_count: 0,
            merkle_root: [0; 32],
            schema_hash: [0; 32],
            signature: [0; 64],
        }
    }

    pub fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&self.format_ver.to_le_bytes());
        out.extend_from_slice(&(HEADER_LEN as u16).to_le_bytes());
        out.extend_from_slice(&self.flags.to_le_bytes());
        out.extend_from_slice(&self.toc_off.to_le_bytes());
        out.extend_from_slice(&self.toc_len.to_le_bytes());
        out.extend_from_slice(&self.toc_raw_len.to_le_bytes());
        out.extend_from_slice(&self.block_count.to_le_bytes());
        out.extend_from_slice(&self.chunk_count.to_le_bytes());
        out.extend_from_slice(&self.merkle_root);
        out.extend_from_slice(&self.schema_hash);
        out.extend_from_slice(&self.signature);
        out.extend_from_slice(&[0u8; 4]);
        debug_assert_eq!(out.len(), HEADER_LEN as usize);
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_LEN as usize {
            return Err("file shorter than the header".into());
        }
        if bytes[0..4] != MAGIC {
            return Err("not a .fnfm package (bad magic)".into());
        }
        let format_ver = u16::from_le_bytes([bytes[4], bytes[5]]);
        if format_ver >> 8 != VERSION >> 8 {
            return Err(format!(
                "unsupported format version {:#06x} (this build reads {:#06x})",
                format_ver, VERSION
            ));
        }
        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&bytes[OFF_MERKLE..OFF_MERKLE + 32]);
        let mut schema_hash = [0u8; 32];
        schema_hash.copy_from_slice(&bytes[OFF_SCHEMA..OFF_SCHEMA + 32]);
        let mut signature = [0u8; 64];
        signature.copy_from_slice(&bytes[OFF_SIG..OFF_SIG + 64]);
        Ok(Self {
            format_ver,
            flags: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            toc_off: u64::from_le_bytes(bytes[12..20].try_into().unwrap()),
            toc_len: u64::from_le_bytes(bytes[20..28].try_into().unwrap()),
            toc_raw_len: u64::from_le_bytes(bytes[28..36].try_into().unwrap()),
            block_count: u32::from_le_bytes(bytes[36..40].try_into().unwrap()),
            chunk_count: u32::from_le_bytes(bytes[40..44].try_into().unwrap()),
            merkle_root,
            schema_hash,
            signature,
        })
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

/// Kind of a block, inferred from the logical path during packing.
pub fn kind_for(path: &str) -> u8 {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "proto" | "pb" => KIND_PROTO,
        "toml" => KIND_MANIFEST,
        "ogg" | "mp3" | "wav" | "flac" => KIND_AUDIO,
        _ => KIND_RAW,
    }
}

/// Formats that are already compressed: zstd would only burn CPU.
pub fn is_precompressed(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "ogg" | "mp3" | "wav" | "png" | "jpg" | "jpeg" | "webp" | "zip" | "gz" | "mp4" | "webm"
    )
}

/// Text-ish formats where zstd 19 pays off.
pub fn is_text(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    matches!(
        ext.as_str(),
        "toml" | "json" | "proto" | "pb" | "txt" | "md" | "xml" | "csv"
    )
}

pub(crate) fn align_up(value: u64, align: u64) -> u64 {
    value.div_ceil(align) * align
}
