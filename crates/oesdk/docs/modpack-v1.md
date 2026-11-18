# `.fnfm` mod package — format v1

Status: **draft, frozen for implementation**. Little-endian throughout.
All offsets are absolute from the start of the file. Unknown `flags` bits or a
`format_ver` major other than 1 must be rejected (fail closed).

## 1. Layout

```
+0      header (176 B)
+128    block data, each block 16-byte aligned
...     TOC (zstd frame, decompresses to BlockTable + StringTable)
...     signature is in the header (no trailing bytes)
```

```
struct Header {            // 176 bytes (172 used, 4 reserved)
  magic:        [u8; 4] = b"FNFM",
  format_ver:   u16 = 1,      // major in high byte, minor in low byte
  header_len:   u16 = 176,
  flags:        u32,          // bit0 signed, bit1 toc_compressed, rest reserved
  toc_off:      u64,
  toc_len:      u64,          // bytes on disk (compressed)
  toc_raw_len:  u64,          // bytes after decompression
  block_count:  u32,
  chunk_count:  u32,
  merkle_root:  [u8; 32],
  schema_hash:  [u8; 32],     // SHA-256 of the proto FileDescriptorSet
  signature:    [u8; 64],     // Ed25519, zero when unsigned
}
```
172 = 4+2+2+4+8*3+4*2+32*2+64; four reserved bytes pad it to 176 so the block area starts 16-byte aligned.

## 2. Block data

Each file is one block. `data_off` is 16-byte aligned; padding bytes are zero.

The algorithm is a byte of the block record (`algo`, byte 7), never a prefix of
the block data: `0 = store`, `1 = zstd`, `2 = zstd-chunked`.

- `store` — bytes are the raw file. Used for already-compressed data: ogg, mp3,
  mp4, png/jpeg when it is imported verbatim.
- `zstd` — one zstd frame. Level is a build choice, not recorded: 19 for proto /
  text / TOML, 9–15 for raw RGBA, `store` otherwise.
- `zstd-chunked` — only for assets that need random access (raw RGBA atlases,
  big JSON). The raw file is cut into `chunk_raw ≤ 256 KiB` pieces, each an
  independent zstd frame; `chunk_count` entries follow the block table:

```
struct Chunk { raw_len: u32, stored_len: u32, stored_off: u64 }   // 16 B, stored_off relative to the block's data_off
```

The last chunk may be smaller. A reader decodes only the chunks it touches.

- Big images are converted to **raw RGBA** at build time (Bevy uploads RGBA
  anyway; PNG decode on a phone costs 30–80 ms per 2048² texture). The block
  name carries the logical path (`assets/foo.png`), the block `flags` carry
  `KIND_IMAGE_RGBA` so the loader knows the bytes are not a PNG.

## 3. TOC

The TOC is a zstd frame (level 19) of:

```
u32 version = 1
u32 block_count
u32 chunk_count
u32 string_bytes
Block[block_count]           // 40 bytes each, sorted by path
Chunk[chunk_count]
u8  strings[string_bytes]    // NUL-separated UTF-8 paths
```

```
struct Block {              // 40 bytes
  path_off:   u32,          // into StringTable
  path_len:   u16,
  kind:       u8,           // 0 proto, 1 image_rgba, 2 audio, 3 raw, 4 manifest
  algo:       u8,           // 0 store, 1 zstd, 2 zstd-chunked
  data_off:   u64,
  stored_len: u64,
  raw_len:    u64,
  crc32c:     u32,          // of the *raw* bytes
  chunk_start: u32,         // index into the global chunk table
}
```

`store` is forced below 64 bytes (zstd's frame header costs more than it
saves) and `zstd-chunked` is chosen for non-text files of 256 KiB or more.

Paths are relative, `/`-separated, NFC, no leading `/`, no `.`/`..` segments,
no backslashes, and must not collide after case folding on case-insensitive
filesystems (the packer rejects such packs).

## 4. Hashing and signature

Leaves are sorted by path bytes:

```
leaf(path, raw) = SHA256(0x00 || path || 0x00 || u64le(len(raw)) || SHA256(raw))
leaf("__toc__") = SHA256(0x00 || "__toc__" || u64le(toc_raw_len) || SHA256(toc_raw))
node(l, r)      = SHA256(0x01 || l || r)          # odd node is promoted unchanged
root            = fold of leaves in order
```

`__toc__` is a synthetic leaf so the table itself cannot be swapped. Signing:

```
signature = Ed25519_sign(sk, b"FNFM1\x00" || merkle_root)
```

`merkle_root` is *not* the hash of the whole file: streaming writers never hold
the pack in memory, and a reader can verify one block before decoding it.

## 5. Reading

1. mmap the file, check magic/`format_ver`/`header_len`.
2. Decompress the TOC, validate `string_bytes` and every `path_off + path_len`
   offset, and that `data_off + stored_len ≤ toc_off`.
3. If signed: rebuild the root from the TOC hashes and verify the signature
   once per launch (cache `(size, mtime, merkle_root)` in the mod cache).
4. On first access of a block: decompress, check `raw_len`, `crc32c`, then
   `SHA256(raw)` against its leaf.
5. Reject when `sum(raw_len) > 2 GiB` or `stored_len * 200 < raw_len`
   (decompression bomb guard).

## 6. Writer rules

- Blocks sorted by path; chunk table ordered by first use.
- `raw_len == stored_len` for `store`; `algo` is derived, not stored twice.
- Never emit a pack with a duplicate path, a directory entry, or an empty file.
- `schema_hash` is written by the compiler that produced the proto blocks; a
  reader with a different hash still loads, but logs a compatibility warning
  and skips unknown proto fields (proto3 semantics).
- `oesdk pack` refuses to pack `target/`, `.git/`, `*.wasm.map`, and files
  above 512 MiB.

## 7. Versioning

- `format_ver` major bump = incompatible reader; minor bump = additive fields
  only (readers must ignore unknown flags).
- v1 reserves: encryption (flag bit 2), per-block signatures (bit 3),
  delta blocks (kind 5).

## 8. Worked example

```
header            0..128
mod.wasm          128..      (store,   1 234 567 B)
manifest.pb       1 234 704  (zstd,      4 096 B → 1 800 B)
cover.rgba        1 236 512  (chunked, 4096×4096×4 = 64 MiB, 256 chunks)
toc               69 208 064 (zstd 19)
```
