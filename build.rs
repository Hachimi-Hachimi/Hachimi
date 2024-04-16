fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" {
        // Link proxy export defs
        let absolute_path = std::fs::canonicalize("src/windows/proxy/exports.def").unwrap();
        println!("cargo:rustc-cdylib-link-arg=/DEF:{}", absolute_path.display());
    }
}