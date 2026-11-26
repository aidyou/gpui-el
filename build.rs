use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn env_path(name: &str) -> io::Result<PathBuf> {
    match env::var_os(name) {
        Some(value) => Ok(PathBuf::from(value)),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{name} is not set"),
        )),
    }
}

fn main() -> io::Result<()> {
    let dir = Path::new("src/icons/element-plus");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "svg")
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            names.push(name.to_owned());
        }
    }
    names.sort_unstable_by(|a, b| a.trim_end_matches(".svg").cmp(b.trim_end_matches(".svg")));
    let root = env_path("CARGO_MANIFEST_DIR")?;
    let out = env_path("OUT_DIR")?;
    let mut file = fs::File::create(out.join("element_plus_icons.rs"))?;
    writeln!(file, "static ELEMENT_PLUS: &[(&str, &[u8])] = &[")?;
    for name in names {
        let path = root.join(dir).join(&name);
        let Some(path) = path.to_str() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("icon path is not UTF-8: {path:?}"),
            ));
        };
        writeln!(
            file,
            "({:?}, include_bytes!({:?})),",
            name.trim_end_matches(".svg"),
            path
        )?;
    }
    writeln!(file, "];")
}
