//! `oesdk` — mod project tool for FNF: Oxide Engine.
//!
//! Single binary: scaffold, validate, and build `.fnfm` packages.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};

use oesdk::abi;
use oesdk::content;
use oesdk::manifest::ModManifest;
use oesdk::pack::{reader::Pack, writer::PackBuilder};

#[derive(Parser)]
#[command(name = "oesdk", about = "FNF: Oxide Engine mod tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a new mod project
    New { dir: PathBuf },
    /// Print the parsed mod.toml
    Inspect { dir: Option<PathBuf> },
    /// Validate the project against the SDK ABI
    Build { dir: Option<PathBuf> },
    /// Print ABI and toolchain information
    Doctor,
    /// Compile content/ and pack a directory into a .fnfm
    Pack {
        dir: PathBuf,
        #[arg(short, long)]
        out: PathBuf,
        /// Ed25519 seed as hex (32 bytes)
        #[arg(long)]
        seed: Option<String>,
    },
    /// List the entries in a package
    List { file: PathBuf },
    /// Verify a package signature
    Verify {
        file: PathBuf,
        #[arg(long = "pub")]
        public: String,
    },
    /// Print the verifying key for a seed
    Pubkey {
        #[arg(long)]
        seed: String,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::New { dir } => cmd_new(&dir),
        Command::Inspect { dir } => cmd_inspect(dir.unwrap_or_else(|| PathBuf::from("."))),
        Command::Build { dir } => cmd_build(dir.unwrap_or_else(|| PathBuf::from("."))),
        Command::Doctor => cmd_doctor(),
        Command::Pack { dir, out, seed } => cmd_pack(&dir, &out, seed.as_deref()),
        Command::List { file } => cmd_list(&file),
        Command::Verify { file, public } => cmd_verify(&file, &public),
        Command::Pubkey { seed } => cmd_pubkey(&seed),
    }
}

fn parse_seed(hex: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (index, byte) in hex.as_bytes().chunks(2).take(32).enumerate() {
        out[index] = u8::from_str_radix(std::str::from_utf8(byte).unwrap_or("00"), 16).unwrap_or(0);
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

fn cmd_pack(dir: &Path, out: &Path, seed: Option<&str>) -> ExitCode {
    let mut compiled = Vec::new();
    let content_dir = dir.join("content");
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
                return ExitCode::FAILURE;
            }
        }
    }
    let mut files = Vec::new();
    if let Err(error) = walk(dir, Path::new(""), &mut files) {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    let mut builder = PackBuilder::new();
    for item in &compiled {
        if let Err(error) = builder.add(item.path.clone(), item.bytes.clone()) {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    }
    for (name, path) in &files {
        if name.starts_with("content/") && name.ends_with(".toml") {
            continue;
        }
        match std::fs::read(path) {
            Ok(bytes) => {
                if let Err(error) = builder.add(name.clone(), bytes) {
                    eprintln!("error: {error}");
                    return ExitCode::FAILURE;
                }
            }
            Err(error) => {
                eprintln!("error: {}: {error}", path.display());
                return ExitCode::FAILURE;
            }
        }
    }
    let key = seed.map(|seed| SigningKey::from_bytes(&parse_seed(seed)));
    match builder.finish(key.as_ref()) {
        Ok(bytes) => match std::fs::write(out, &bytes) {
            Ok(()) => {
                println!(
                    "packed {} file(s) -> {} ({} bytes)",
                    files.len(),
                    out.display(),
                    bytes.len()
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {}: {error}", out.display());
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_list(file: &Path) -> ExitCode {
    let bytes = match std::fs::read(file) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("error: {}: {error}", file.display());
            return ExitCode::FAILURE;
        }
    };
    let pack = match Pack::parse(&bytes) {
        Ok(pack) => pack,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    for entry in pack.entries() {
        println!(
            "{:>10} {:>10} {} {}",
            entry.raw_len, entry.stored_len, entry.kind, entry.path
        );
    }
    ExitCode::SUCCESS
}

fn cmd_verify(file: &Path, public: &str) -> ExitCode {
    let bytes = match std::fs::read(file) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("error: {}: {error}", file.display());
            return ExitCode::FAILURE;
        }
    };
    let pack = match Pack::parse(&bytes) {
        Ok(pack) => pack,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let key = VerifyingKey::from_bytes(&parse_seed(public)).expect("bad key");
    match pack.verify_signature(&key) {
        Ok(()) => {
            println!("ok: signature and Merkle root verified");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("verify failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_pubkey(seed: &str) -> ExitCode {
    let key = SigningKey::from_bytes(&parse_seed(seed));
    println!("{}", hex(key.verifying_key().as_bytes()));
    ExitCode::SUCCESS
}

fn cmd_new(dir: &Path) -> ExitCode {
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-mod")
        .to_string();
    let id: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    if dir.join("mod.toml").exists() {
        eprintln!("error: {} already contains mod.toml", dir.display());
        return ExitCode::from(1);
    }
    if let Err(err) = fs::create_dir_all(dir.join("src")) {
        eprintln!("error: cannot create {}: {err}", dir.display());
        return ExitCode::from(1);
    }

    let manifest = format!(
        r#"id = "{id}"
name = "{name}"
version = "0.1.0"
api = "{api}"
entry = "__oesdk_make_mod"
capabilities = ["camera", "sprites", "audio", "rng"]
callbacks = ["on-create", "on-beat", "on-note-hit"]

[metadata]
author = "you"
description = ""
"#,
        api = abi::ABI_VERSION
    );
    let lib = r#"use oesdk::prelude::*;

#[derive(Default)]
pub struct MyMod;

impl Mod for MyMod {
    fn on_create(&mut self, ctx: &mut Ctx<'_>) {
        ctx.log(LogLevel::Info, "hello from my mod");
    }

    fn on_note_hit(&mut self, ctx: &mut Ctx<'_>, hit: &Hit) {
        if hit.judgement == Judgement::Sick {
            ctx.camera().shake(0.05, 0.2);
        }
    }
}

oesdk::register_mod!(MyMod);
"#;

    if let Err(err) = fs::write(dir.join("mod.toml"), manifest) {
        eprintln!("error: cannot write mod.toml: {err}");
        return ExitCode::from(1);
    }
    if let Err(err) = fs::write(dir.join("src/lib.rs"), lib) {
        eprintln!("error: cannot write src/lib.rs: {err}");
        return ExitCode::from(1);
    }
    println!("created {} (id `{id}`)", dir.display());
    ExitCode::SUCCESS
}

fn load_manifest(dir: &Path) -> Result<ModManifest, ExitCode> {
    let path = dir.join("mod.toml");
    let src = fs::read_to_string(&path).map_err(|err| {
        eprintln!("error: cannot read {}: {err}", path.display());
        ExitCode::from(1)
    })?;
    ModManifest::parse(&src).map_err(|err| {
        eprintln!("error: {err}");
        ExitCode::from(1)
    })
}

fn cmd_inspect(dir: PathBuf) -> ExitCode {
    let manifest = match load_manifest(&dir) {
        Ok(m) => m,
        Err(code) => return code,
    };
    println!("mod.toml ({})", dir.join("mod.toml").display());
    println!("  id           {}", manifest.id);
    println!("  name         {}", manifest.name);
    println!("  version      {}", manifest.version);
    println!("  api          {}", manifest.api);
    println!("  entry        {}", manifest.entry);
    println!("  capabilities {:?}", manifest.capabilities);
    println!("  callbacks    {:?}", manifest.callbacks);
    ExitCode::SUCCESS
}

fn cmd_build(dir: PathBuf) -> ExitCode {
    let manifest = match load_manifest(&dir) {
        Ok(m) => m,
        Err(code) => return code,
    };

    let mut errors = 0usize;
    if manifest.api != abi::ABI_VERSION {
        eprintln!(
            "error: api `{}` does not match SDK ABI `{}`",
            manifest.api,
            abi::ABI_VERSION
        );
        errors += 1;
    }
    for cb in &manifest.callbacks {
        if !abi::is_known_callback(cb) {
            eprintln!("error: unknown callback `{cb}`");
            errors += 1;
        }
    }
    if !dir.join("src/lib.rs").exists() {
        eprintln!("error: src/lib.rs not found");
        errors += 1;
    }
    if errors > 0 {
        eprintln!("\n{errors} error(s)");
        return ExitCode::from(1);
    }

    println!("checked {} v{}", manifest.id, manifest.version);
    println!("  entry      {}", manifest.entry);
    println!("  callbacks  {:?}", manifest.callbacks);
    println!("  wasm build : TODO (wit-bindgen + cargo component)");
    println!("  package    : TODO (.fnfm + zstd + Ed25519)");
    ExitCode::SUCCESS
}

fn cmd_doctor() -> ExitCode {
    println!("oesdk     {}", env!("CARGO_PKG_VERSION"));
    println!("ABI       {}", abi::ABI_VERSION);
    println!("callbacks {}", abi::CALLBACKS.join(", "));
    println!("license   MIT OR Apache-2.0");
    ExitCode::SUCCESS
}
