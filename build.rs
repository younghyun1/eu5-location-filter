fn main() {
    if let Err(error) = stage_embedded_assets() {
        eprintln!("failed to stage embedded data: {error}");
        std::process::exit(1);
    }
    let configuration =
        slint_build::CompilerConfiguration::new().with_style("fluent-dark".to_owned());
    if let Err(error) = slint_build::compile_with_config("ui/app.slint", configuration) {
        eprintln!("failed to compile Slint UI: {error}");
        std::process::exit(1);
    }
}

fn stage_embedded_assets() -> Result<(), std::io::Error> {
    let Some(output) = std::env::var_os("OUT_DIR") else {
        return Err(std::io::Error::other("Cargo did not set OUT_DIR"));
    };
    let output = std::path::PathBuf::from(output);
    for name in ["eu5-locations.bitcode.zst", "eu5-indexes.bitcode.zst"] {
        let source = std::path::Path::new("assets").join(name);
        let destination = output.join(name);
        println!("cargo:rerun-if-changed={}", source.display());
        if !source.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("required committed bundle is missing: {}", source.display()),
            ));
        }
        std::fs::copy(source, destination)?;
    }
    Ok(())
}
