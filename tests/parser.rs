#[test]
fn parse_wasm() -> Result<(), std::io::Error> {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "input.wasm"]
        .iter()
        .collect();

    use std::fs::File;
    File::open(path)?;

    Ok(())
}
