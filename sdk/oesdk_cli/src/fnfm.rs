//! Minimal `.fnfm` tool: pack / verify / list.

use ed25519_dalek::{SigningKey, VerifyingKey};
use oesdk::content;
use oesdk::pack::{reader::Pack, writer::PackBuilder};
use std::path::{Path, PathBuf};

fn usage() -> ! {
    eprintln!("usage: fnfm <pack DIR -o OUT [--seed HEX] | verify FILE --pub HEX | list FILE>");
    std::process::exit(2)
}

fn parse_seed(hex: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (index, byte) in hex.as_bytes().chunks(2).take(32).enumerate() {
        out[index] = u8::from_str_radix(std::str::from_utf8(byte).unwrap(), 16).unwrap();
    }
    out
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn walk(root: &Path, prefix: &Path, out: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(root)?.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let rel = prefix.join(entry.file_name());
        if path.is_dir() {
            walk(&path, &rel, out)?;
        } else {
            out.push((rel.to_string_lossy().replace('\\', "/"), path));
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    match args[0].as_str() {
        "pack" => {
            let dir = args.get(1).unwrap_or_else(|| usage());
            let out = args
                .iter()
                .position(|arg| arg == "-o")
                .and_then(|index| args.get(index + 1))
                .unwrap_or_else(|| usage());
            let seed = args
                .iter()
                .position(|arg| arg == "--seed")
                .and_then(|index| args.get(index + 1))
                .map(|hex| parse_seed(hex));

            let mut compiled = Vec::new();
            let content_dir = Path::new(dir).join("content");
            if content_dir.is_dir() {
                match content::compile(&content_dir) {
                    Ok(items) => {
                        for item in items {
                            println!("compiled {} ({} bytes)", item.path, item.bytes.len());
                            compiled.push(item);
                        }
                    }
                    Err(error) => {
                        eprintln!("content error: {error}");
                        std::process::exit(1);
                    }
                }
            }

            let mut files = Vec::new();
            walk(Path::new(dir), Path::new(""), &mut files).expect("walk failed");
            let mut builder = PackBuilder::new();
            for item in &compiled {
                builder
                    .add(item.path.clone(), item.bytes.clone())
                    .expect("add failed");
            }
            for (name, path) in &files {
                // TOML sources are compiled above; ship only the protobuf.
                if name.starts_with("content/") && name.ends_with(".toml") {
                    continue;
                }
                builder
                    .add(name.clone(), std::fs::read(path).expect("read failed"))
                    .expect("add failed");
            }
            let key = seed.map(|seed| SigningKey::from_bytes(&seed));
            let bytes = builder.finish(key.as_ref()).expect("pack failed");
            std::fs::write(out, &bytes).expect("write failed");
            println!(
                "packed {} file(s) -> {} ({} bytes)",
                files.len(),
                out,
                bytes.len()
            );
        }
        "pubkey" => {
            let seed = args
                .iter()
                .position(|arg| arg == "--seed")
                .and_then(|index| args.get(index + 1))
                .unwrap_or_else(|| usage());
            let key = SigningKey::from_bytes(&parse_seed(seed));
            println!("{}", hex(key.verifying_key().as_bytes()));
        }
        "list" => {
            let bytes = std::fs::read(args.get(1).unwrap_or_else(|| usage())).expect("read failed");
            let pack = Pack::parse(&bytes).expect("parse failed");
            for entry in pack.entries() {
                println!(
                    "{:>10} {:>10} {} {}",
                    entry.raw_len, entry.stored_len, entry.kind, entry.path
                );
            }
        }
        "verify" => {
            let bytes = std::fs::read(args.get(1).unwrap_or_else(|| usage())).expect("read failed");
            let pack = Pack::parse(&bytes).expect("parse failed");
            let hex = args
                .iter()
                .position(|arg| arg == "--pub")
                .and_then(|index| args.get(index + 1))
                .unwrap_or_else(|| usage());
            let key = VerifyingKey::from_bytes(&parse_seed(hex)).expect("bad key");
            match pack.verify_signature(&key) {
                Ok(()) => println!("ok: signature and Merkle root verified"),
                Err(error) => {
                    eprintln!("verify failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        _ => usage(),
    }
}
