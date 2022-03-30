use std::fs::File;

use wario::parser::Result;
use wario::wasm;

#[test]
fn parse_wasm() -> Result<()> {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "mandelbrot.wasm"]
        .iter()
        .collect();

    let file = File::open(path);
    let mut file = match file {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open file: {}", err)),
    };

    let module = wasm::Module::parse(&mut file)?;
    println!("{:#?}", module);

    Ok(())
}
