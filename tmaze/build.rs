use boml::prelude::*;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env::current_dir, fmt::Display, fs, io};
use thiserror::Error;

#[derive(Debug)]
#[allow(dead_code)]
struct BomlParseError {
    kind: TomlErrorKind,
    src: String,
}
impl Display for BomlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for BomlParseError {}
impl From<TomlError<'_>> for BomlParseError {
    fn from(value: TomlError) -> Self {
        Self {
            kind: value.kind,
            src: value.src.as_str().to_string(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum BomlGetError {
    InvalidKey,
    TypeMismatch(TomlValueType),
}
impl Display for BomlGetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for BomlGetError {}
impl From<TomlGetError<'_, '_>> for BomlGetError {
    fn from(value: TomlGetError) -> Self {
        match value {
            TomlGetError::InvalidKey => Self::InvalidKey,
            TomlGetError::TypeMismatch(_, toml_value_type) => Self::TypeMismatch(toml_value_type),
        }
    }
}

#[derive(Error, Debug)]
enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("BOML parse error: {0}")]
    Boml(#[from] BomlParseError),
    #[error("TOML get error: {0}")]
    TomlGet(#[from] BomlGetError),
    #[error("Fmt error")]
    Fmt(#[from] std::fmt::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub type Rgb = [u8; 3];

#[derive(Debug)]
struct Scheme {
    name: String,
    primary_fg: Rgb,
    primary_bg: Rgb,
    black: Rgb,     // dark grey
    dark_grey: Rgb, // grey
    red: Rgb,
    dark_red: Rgb,
    green: Rgb,
    dark_green: Rgb,
    yellow: Rgb,
    dark_yellow: Rgb,
    blue: Rgb,
    dark_blue: Rgb,
    magenta: Rgb,
    dark_magenta: Rgb,
    cyan: Rgb,
    dark_cyan: Rgb,
    white: Rgb,
    grey: Rgb,
}

impl Scheme {
    fn new(name: String) -> Self {
        Self {
            name,
            // primary colors
            primary_fg: [0x18, 0x18, 0x18],
            primary_bg: [0xd8, 0xd8, 0xd8],
            // normal colors
            black: [0x18, 0x18, 0x18],
            dark_red: [0xac, 0x42, 0x42],
            dark_green: [0x90, 0xa9, 0x59],
            dark_yellow: [0xf4, 0xbf, 0x75],
            dark_blue: [0x6a, 0x9f, 0xb5],
            dark_magenta: [0xaa, 0x75, 0x9f],
            dark_cyan: [0x75, 0xb5, 0xaa],
            grey: [0xd8, 0xd8, 0xd8],
            // bright colors
            dark_grey: [0x6b, 0x6b, 0x6b],
            red: [0xc5, 0x55, 0x55],
            green: [0xaa, 0xc4, 0x74],
            yellow: [0xfe, 0xca, 0x88],
            blue: [0x82, 0xb8, 0xc8],
            magenta: [0xc2, 0x8c, 0xb8],
            cyan: [0x93, 0xd3, 0xc3],
            white: [0xf8, 0xf8, 0xf8],
        }
    }

    fn iter_fields(
        &mut self,
    ) -> impl Iterator<Item = (&'static str, (&'static str, &'static str), &mut Rgb)> {
        [
            (
                "primary_fg",
                ("primary", "foreground"),
                &mut self.primary_fg,
            ),
            (
                "primary_bg",
                ("primary", "background"),
                &mut self.primary_bg,
            ),
            ("black", ("normal", "black"), &mut self.black),
            ("dark_grey", ("bright", "black"), &mut self.dark_grey),
            ("red", ("bright", "red"), &mut self.red),
            ("dark_red", ("normal", "red"), &mut self.dark_red),
            ("green", ("bright", "green"), &mut self.green),
            ("dark_green", ("normal", "green"), &mut self.dark_green),
            ("yellow", ("bright", "yellow"), &mut self.yellow),
            ("dark_yellow", ("normal", "yellow"), &mut self.dark_yellow),
            ("blue", ("bright", "blue"), &mut self.blue),
            ("dark_blue", ("normal", "blue"), &mut self.dark_blue),
            ("magenta", ("bright", "magenta"), &mut self.magenta),
            (
                "dark_magenta",
                ("normal", "magenta"),
                &mut self.dark_magenta,
            ),
            ("cyan", ("bright", "cyan"), &mut self.cyan),
            ("dark_cyan", ("normal", "cyan"), &mut self.dark_cyan),
            ("white", ("bright", "white"), &mut self.white),
            ("grey", ("normal", "white"), &mut self.grey),
        ]
        .into_iter()
    }
}

fn parse_rgb(s: &str) -> Result<Rgb, MyError> {
    let error = || MyError::Other(format!("Invalid RGB string: {}", s));
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return Err(error());
    }
    let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| error())?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| error())?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| error())?;
    Ok([r, g, b])
}

fn extract_scheme(filename: &OsStr, toml: &TomlTable) -> Result<Scheme, MyError> {
    let cerr = <BomlGetError as From<TomlGetError>>::from;

    let colors = toml.get_table("colors").map_err(cerr)?;

    let mut scheme = Scheme::new(
        filename
            .to_str()
            .ok_or(MyError::Other("Invalid filename".into()))?
            .to_string(),
    );

    for (_, (section, color), field) in scheme.iter_fields() {
        if let Ok(color) = colors.get_table(section).and_then(|t| t.get_string(color)) {
            *field = parse_rgb(color)?;
        }
    }

    Ok(scheme)
}

fn write_scheme_case(buf: &mut String, scheme: &mut Scheme) -> Result<(), MyError> {
    writeln!(buf, "    \"{}\" => Self {{", scheme.name)?;
    for (field_name, _, color) in scheme.iter_fields() {
        writeln!(
            buf,
            "        {field_name}: ({}, {}, {}),",
            color[0], color[1], color[2]
        )?;
    }
    buf.push_str("    },\n");

    Ok(())
}

fn write_scheme_name(scheme_name_array: &mut String, scheme: &Scheme) {
    if !scheme_name_array.ends_with('[') {
        scheme_name_array.push_str(", ");
    }
    write!(scheme_name_array, "\"{}\"", scheme.name).unwrap();
}

fn process_schemes(out_dir: &Path) -> Result<(), MyError> {
    let schemes_dir = current_dir()?.join("assets/forgen/schemes");
    let schemes = fs::read_dir(&schemes_dir)?;

    let mut scheme_name_array: String = "[".into();

    let mut match_code: String = "// This file is generated by build.
// Do not edit this file directly.

match scheme_name {
"
    .into();

    for scheme_entry in schemes {
        let scheme_entry = scheme_entry?;
        eprintln!("Processing scheme: {:?}", scheme_entry);
        let content = fs::read_to_string(scheme_entry.path())?;
        let toml = boml::parse(&content).map_err(BomlParseError::from)?;
        let mut scheme = extract_scheme(
            scheme_entry
                .path()
                .file_stem()
                .ok_or(MyError::Other("Invalid filename".into()))?,
            &toml,
        )?;
        write_scheme_case(&mut match_code, &mut scheme)?;

        write_scheme_name(&mut scheme_name_array, &scheme);
    }

    match_code.push_str(r#"    _ => panic!("Unknown terminal color scheme: {}", scheme_name),"#);
    match_code.push_str("\n}\n");
    scheme_name_array.push(']');

    let out_match = out_dir.join("schemes_match.in");
    fs::write(&out_match, match_code)?;
    let out_names = out_dir.join("schemes_names.in");
    fs::write(&out_names, scheme_name_array)?;

    println!("cargo::rerun-if-changed=assets/forgen/schemes");

    Ok(())
}

fn main() -> Result<(), MyError> {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").ok_or(MyError::Other(
        "OUT_DIR environment variable not set".into(),
    ))?);

    process_schemes(&out_dir)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
