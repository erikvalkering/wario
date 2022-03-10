use std::fs::File;

use wario;

#[test]
fn parse_wasm() -> wario::Result<()> {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "input.wasm"]
        .iter()
        .collect();

    let file = File::open(path);
    let mut file = match file {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open file: {}", err)),
    };

    let module = wario::Module::parse(&mut file)?;
    println!("{:#?}", module);

    Ok(())
}
