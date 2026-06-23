fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS")
        .unwrap_or_default()
        .to_lowercase();

    if target_os == "windows" {
        println!("cargo::rustc-link-arg-bin=serverare=/MANIFEST:EMBED");
        println!("cargo::rustc-link-arg-bin=serverare=/MANIFESTUAC:level=\'requireAdministrator\'");
    }
}
