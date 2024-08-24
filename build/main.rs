mod instructions_template;
mod static_string_indexing;

use instructions_template::generate_instructions_rs;
use static_string_indexing::index_static_strings;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn find_prolog_files(path_prefix: &str, current_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut libraries = vec![];

    let entries = match current_dir.read_dir() {
        Ok(entries) => entries,
        Err(_) => return libraries,
    };

    for entry in entries.filter_map(Result::ok).map(|e| e.path()) {
        if entry.is_dir() {
            if let Some(file_name) = entry.file_name() {
                let file_name = file_name.to_str().unwrap();
                let new_path_prefix = format!("{path_prefix}{file_name}/");
                let new_libs = find_prolog_files(&new_path_prefix, &entry);
                libraries.extend(new_libs);
            }
        } else if entry.is_file() {
            let ext = std::ffi::OsStr::new("pl");
            if entry.extension() == Some(ext) {
                let name = entry.file_stem().unwrap().to_str().unwrap();
                let lib_name = format!("{path_prefix}{name}");

                libraries.push((lib_name, entry));
            }
        }
    }

    libraries
}

fn main() {
    let has_rustfmt = Command::new("rustfmt")
        .arg("--version")
        .stdin(Stdio::inherit())
        .status()
        .is_ok();

    if !has_rustfmt {
        println!("Failed to run rustfmt, will skip formatting generated files.")
    }

    generate_c_bindings();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("libraries.rs");

    let mut libraries = File::create(dest_path).unwrap();
    let lib_path = Path::new("src").join("lib");

    let constants = find_prolog_files("", &lib_path);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir_path: &Path = out_dir.as_ref();
    let manifest_dir = &std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir_path: &Path = manifest_dir.as_ref();

    let prefix: PathBuf = if let Ok(diff) = out_dir_path.strip_prefix(manifest_dir_path) {
        let mut path = PathBuf::from(".");
        for comp in diff.components() {
            match comp {
                std::path::Component::Normal(_) => path.push(".."),
                std::path::Component::CurDir => (),
                std::path::Component::Prefix(_)
                | std::path::Component::RootDir
                | std::path::Component::ParentDir => {
                    path = manifest_dir_path.to_path_buf();
                    break;
                }
            }
        }
        path
    } else {
        manifest_dir_path.to_path_buf()
    };

    writeln!(libraries, "{{").unwrap();
    for (name, lib_path) in constants {
        let path: PathBuf = prefix.join(lib_path);
        writeln!(libraries, "m.insert(\"{name}\", include_str!({path:?}));").unwrap();
    }
    writeln!(libraries, "}}").unwrap();

    let instructions_path = Path::new(&out_dir).join("instructions.rs");
    let mut instructions_file = File::create(&instructions_path).unwrap();

    let quoted_output = generate_instructions_rs();

    instructions_file
        .write_all(quoted_output.to_string().as_bytes())
        .unwrap();

    if has_rustfmt {
        format_generated_file(instructions_path.as_path());
    }

    let static_atoms_path = Path::new(&out_dir).join("static_atoms.rs");
    let mut static_atoms_file = File::create(&static_atoms_path).unwrap();

    let quoted_output = index_static_strings(&instructions_path);

    static_atoms_file
        .write_all(quoted_output.to_string().as_bytes())
        .unwrap();

    if has_rustfmt {
        format_generated_file(static_atoms_path.as_path());
    }

    println!("cargo:rerun-if-changed=src/");
}

fn generate_c_bindings() {
    println!("cargo:rerun-if-changed=.cbindgen.toml");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let headers_dir = Path::new(&manifest_dir).join("docs/shared_library/libscryer_prolog.h");
    let config =
        cbindgen::Config::from_file(".cbindgen.toml").unwrap_or(cbindgen::Config::default());

    match cbindgen::Builder::new()
        .with_crate(manifest_dir)
        .with_config(config)
        .generate()
    {
        Ok(bindings) => {
            bindings.write_to_file(headers_dir);
        }
        Err(err) => {
            println!("cargo:warning=Failed to generate C bindings: {err}");
        }
    }
}

fn format_generated_file(path: &Path) {
    Command::new("rustfmt")
        .arg(path.as_os_str())
        .spawn()
        .unwrap_or_else(|err| {
            panic!(
                "{}: rustfmt was detected as available, but failed to format generated file '{}'",
                err,
                path.display()
            );
        })
        .wait()
        .unwrap();
}
