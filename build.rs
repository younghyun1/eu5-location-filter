fn main() {
    let configuration =
        slint_build::CompilerConfiguration::new().with_style("fluent-dark".to_owned());
    if let Err(error) = slint_build::compile_with_config("ui/app.slint", configuration) {
        eprintln!("failed to compile Slint UI: {error}");
        std::process::exit(1);
    }
}
