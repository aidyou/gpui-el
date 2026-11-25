use std::{env, fs, io::Write, path::Path};

fn main() -> std::io::Result<()> {
    let dir = Path::new("src/icons/element-plus");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "svg") {
            names.push(path.file_name().unwrap().to_str().unwrap().to_owned());
        }
    }
    names.sort_unstable_by(|a, b| a.trim_end_matches(".svg").cmp(b.trim_end_matches(".svg")));
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out = env::var_os("OUT_DIR").unwrap();
    let mut file = fs::File::create(Path::new(&out).join("element_plus_icons.rs"))?;
    writeln!(file, "static ELEMENT_PLUS: &[(&str, &[u8])] = &[")?;
    for name in names {
        let path = Path::new(&root).join(dir).join(&name);
        writeln!(
            file,
            "({:?}, include_bytes!({:?})),",
            name.trim_end_matches(".svg"),
            path.to_str().unwrap()
        )?;
    }
    writeln!(file, "];")
}
