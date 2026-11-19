//! `oesdk` — mod project tool for FNF: Oxide Engine.
//!
//! Scaffolding lives here so the contract (`oesdk::abi`, `wit/oesdk.wit`) and
//! the tool that validates against it cannot drift apart.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oesdk::abi;
use oesdk::manifest::ModManifest;

const USAGE: &str = "\
oesdk — FNF: Oxide Engine mod tool

USAGE:
    oesdk <command> [options]

COMMANDS:
    new <dir>         scaffold a new mod project
    inspect [dir]     print the parsed mod.toml
    build [dir]       validate + plan a build (wasm compilation: TODO)
    pack [dir]        package into .fnfm (TODO)
    sign [dir]        sign a package with Ed25519 (TODO)
    verify <file>     verify a package signature (TODO)
    doctor            print ABI/toolchain information
    help              show this message
";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(command) = args.first().map(String::as_str) else {
        print!("{USAGE}");
        return ExitCode::from(2);
    };
    let rest: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();

    match command {
        "new" => match rest.first() {
            Some(dir) => cmd_new(Path::new(dir)),
            None => usage_error("`new` needs a directory"),
        },
        "inspect" => cmd_inspect(project_dir(rest.first().copied())),
        "build" => cmd_build(project_dir(rest.first().copied())),
        "pack" => not_yet("pack"),
        "sign" => not_yet("sign"),
        "verify" => not_yet("verify"),
        "doctor" => cmd_doctor(),
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        other => usage_error(&format!("unknown command `{other}`")),
    }
}

fn usage_error(msg: &str) -> ExitCode {
    eprintln!("error: {msg}\n");
    eprint!("{USAGE}");
    ExitCode::from(2)
}

fn not_yet(cmd: &str) -> ExitCode {
    eprintln!("oesdk: `{cmd}` is not implemented yet (skeleton only)");
    ExitCode::from(3)
}

fn project_dir(arg: Option<&str>) -> PathBuf {
    arg.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
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
