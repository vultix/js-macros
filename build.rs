use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

macro_rules! build_script_var {
    ($name:literal) => {
        std::env::var($name).unwrap_or_else(|_| panic!("Missing build script var {}", $name))
    };
}

fn main() -> Result<()> {
    if std::env::var("DOCS_RS").is_ok() {
        return Ok(());
    }

    let out_dir = build_script_var!("OUT_DIR");
    let macros_dir = out_dir
        .split("target")
        .next()
        .expect("OUT_DIR should contain target");

    let macros_dir = PathBuf::from(macros_dir).join("js-macros");
    println!("cargo:rerun-if-changed={}", macros_dir.to_string_lossy());

    let js_macro_dir = PathBuf::from(build_script_var!("CARGO_MANIFEST_DIR"));
    let js_macro_lib = js_macro_dir.join("src/lib.rs");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&js_macro_lib)
        .with_context(|| format!("IO Error when opening js-macro lib file {:?}", js_macro_lib))?;

    let mut writer = BufWriter::new(file);
    let write_lib_context =
        || format!("IO Error when writing js-macro lib file {:?}", js_macro_lib);
    writer.write_all(HEADER).with_context(write_lib_context)?;

    let regex = JsMacro::regex();

    let file_iter = macros_dir
        .read_dir()
        .with_context(|| format!("IO err when reading js-macros dir {:?}", macros_dir))?;

    for file in file_iter {
        let path = file
            .with_context(|| format!("IO err when reading js-macros dir {:?}", macros_dir))?
            .path();

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("IO err reading file {:?}", path))?;

        let js_macro = JsMacro::try_from_file(path.clone(), &contents, &regex)
            .with_context(|| format!("Failed parsing js-macro file {:?}", path))?;
        if let Some(js_macro) = js_macro {
            writer
                .write_all(js_macro.to_rust_macro().as_bytes())
                .with_context(write_lib_context)?;
        }
    }

    Ok(())
}

static HEADER: &[u8] = br##"
#![doc = include_str!("../README.md")]

// AUTO-GENERATED LIB FILE - DON'T MODIFY

extern crate proc_macro;

use proc_macro::{LexError, TokenStream};
use std::process::Command;
use proc_macro_error::*;

fn run_node_macro(
    macro_input: TokenStream,
    macro_args: Option<TokenStream>,
    node_script_path: &str,
) -> Result<TokenStream, LexError> {
    let output = Command::new("node")
        .arg("-e")
        .arg(format!(
            r"
                MACRO_INPUT = {:?};
                MACRO_ARGUMENTS = {:?};
                MACRO_OUTPUT = '';
                require({:?});
                console.log('MACRO_OUTPUT: ' + MACRO_OUTPUT);
            ",
            macro_input.to_string(),
            macro_args.map(|a| a.to_string()).unwrap_or_default(),
            node_script_path
        ))
        .output()
        .unwrap_or_else(|e| {
            if let std::io::ErrorKind::NotFound = e.kind() {
                panic!("No `node` command found in PATH");
            }

            panic!("IO Error running JS macro: {}", e)
        });

    if !output.status.success() {
        emit_call_site_error!(format!("Error occurred running JS macro:\n{}\n", String::from_utf8_lossy(&output.stderr)));
        return Ok(Default::default());
    }

    let macro_output = std::str::from_utf8(&output.stdout)
        .expect("JS Macro had unexpected non-utf8 output")
        .split("MACRO_OUTPUT: ")
        .nth(1)
        .expect("JS Macro unexpectedly missing macro output");

    macro_output.parse()
}
"##;

struct JsMacro<'a> {
    path: PathBuf,
    name: &'a str,
    macro_type: MacroType<'a>,
}

impl<'a> JsMacro<'a> {
    fn regex() -> Regex {
        regex::RegexBuilder::new(
            r#"
        # JS_MACRO comment
        ^//!\s*JS_MACRO:\s*

        # Macro Type
        (?P<macro_type>\w+?)

        # Macro Name
        \(
            (?P<name>.*?)
        \)\s*

        # Optional attributes list
        (?:
            attributes\(
                (?P<attributes>.*?)
            \)
        )?\s*$
    "#,
        )
        .multi_line(true)
        .ignore_whitespace(true)
        .build()
        .unwrap()
    }

    fn try_from_file(mut path: PathBuf, contents: &'a str, regex: &Regex) -> Result<Option<Self>> {
        let captures = match regex.captures(contents) {
            Some(c) => c,
            None => return Ok(None),
        };

        let macro_type = captures.name("macro_type").unwrap().as_str();
        let name = captures.name("name").unwrap().as_str();
        let attributes = captures.name("attributes").map(|m| m.as_str());

        let macro_type = match (macro_type, attributes) {
            ("derive", Some(attributes)) => MacroType::Derive { attributes },
            ("derive", None) => MacroType::Derive { attributes: "" },
            ("attribute", None) => MacroType::Attribute,
            ("function", None) => MacroType::Function,
            (macro_type @ ("attribute" | "function"), Some(attributes)) => {
                return Err(anyhow!(
                    "Macro type {} does not support helper attributes ({})",
                    macro_type,
                    attributes
                ))
            }
            (other, _) => {
                return Err(anyhow!(
                    "Unexpected macro type: `{}`. Should be one of: derive, attribute, function",
                    other
                ))
            }
        };

        let is_typescript = path.extension().map(|ext| ext == "ts").unwrap_or_default();
        if is_typescript {
            path = compile_typescript(&path)?;
        }

        Ok(Some(JsMacro {
            name,
            path,
            macro_type,
        }))
    }

    fn to_rust_macro(&self) -> String {
        match self.macro_type {
            MacroType::Derive { attributes } => format!(
                "#[proc_macro_error]
                #[proc_macro_derive({}, attributes({}))]
                #[allow(non_snake_case)] #[doc(hidden)]
                pub fn {}(input: TokenStream) -> TokenStream {{
                    run_node_macro(input, None, {:?}).unwrap()
                }}",
                self.name, attributes, self.name, self.path
            ),
            MacroType::Function => format!(
                "#[proc_macro_error]
                #[proc_macro]
                #[allow(non_snake_case)] #[doc(hidden)]
                pub fn {}(input: TokenStream) -> TokenStream {{
                    run_node_macro(input, None, {:?}).unwrap()
                }}",
                self.name, self.path
            ),
            MacroType::Attribute => format!(
                "#[proc_macro_error]
                #[proc_macro_attribute]
                #[allow(non_snake_case)] #[doc(hidden)]
                pub fn {}(args: TokenStream, input: TokenStream) -> TokenStream {{
                    run_node_macro(input, Some(args), {:?}).unwrap()
                }}",
                self.name, self.path
            ),
        }
    }
}

fn compile_typescript(path: &Path) -> Result<PathBuf> {
    let out_dir = build_script_var!("OUT_DIR");
    let file_name = path
        .file_name()
        .expect("Tried compiling non-ts file")
        .to_string_lossy()
        .into_owned();

    let output = Command::new("tsc")
        .arg(path)
        .args(&[
            "--outDir",
            &out_dir,
            "--allowJs",
            "--module",
            "commonjs",
            "--target",
            "esnext",
            "--importsNotUsedAsValues",
            "preserve",
            "--pretty",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| {
            if let std::io::ErrorKind::NotFound = e.kind() {
                anyhow!("No `tsc` command found in PATH");
            }

            anyhow!(e)
        })?;

    if !output.status.success() {
        return Err(anyhow!(
            "TS Compilation error:\n{}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }

    Ok(PathBuf::from(out_dir).join(file_name.replace(".ts", ".js")))
}

enum MacroType<'a> {
    Derive { attributes: &'a str },
    Function,
    Attribute,
}
