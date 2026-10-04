//! Embeds what the command line shares with the app: the interface's
//! dictionaries (`src/lib/locales/*.ts`, so a message reads the same in the
//! terminal as on screen) and the bundled templates (`experiments/templates`).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[path = "src/extract.rs"]
mod extract;

/// The languages of the interface, as in `src/lib/locales/index.ts` (a test in
/// `src/i18n.rs` keeps the two lists equal).
const LANGUAGES: [&str; 11] = ["en", "ru", "es", "fr", "de", "pt", "zh", "ja", "ko", "hi", "ar"];

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("..");
    let mut out = String::new();

    let locales = root.join("src/lib/locales");
    let mut english: Vec<String> = Vec::new();
    for lang in LANGUAGES {
        let path = locales.join(format!("{lang}.ts"));
        println!("cargo:rerun-if-changed={}", path.display());
        let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let entries = extract::entries(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let mut keys: Vec<String> = entries.iter().map(|(key, _)| key.clone()).collect();
        keys.sort();
        if lang == "en" {
            english = keys;
        } else if keys != english {
            panic!("{}: not the same keys as en.ts", path.display());
        }
        writeln!(out, "pub static {}: &[(&str, &str)] = &[", lang.to_uppercase()).unwrap();
        for (key, text) in &entries {
            writeln!(out, "    ({key:?}, {text:?}),").unwrap();
        }
        writeln!(out, "];").unwrap();
    }

    let templates = root.join("experiments/templates");
    println!("cargo:rerun-if-changed={}", templates.display());
    let mut files: Vec<PathBuf> = std::fs::read_dir(&templates)
        .unwrap_or_else(|error| panic!("{}: {error}", templates.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "json"))
        .collect();
    files.sort();
    writeln!(out, "pub static TEMPLATES: &[(&str, &str)] = &[").unwrap();
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
        let name = file.file_stem().unwrap().to_string_lossy();
        let text = std::fs::read_to_string(file).unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        writeln!(out, "    ({name:?}, {:?}),", text.replace("\r\n", "\n")).unwrap();
    }
    writeln!(out, "];").unwrap();

    let target = Path::new(&std::env::var("OUT_DIR").unwrap()).join("embedded.rs");
    std::fs::write(target, out).unwrap();
}
