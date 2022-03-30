use std::fs::File;

use wario::parser::Result;
use wario::wasm;

fn open_file(filename: &str) -> File {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", filename]
        .iter()
        .collect();

    match File::open(path) {
        Ok(file) => file,
        Err(err) => panic!("Unable to open file: {}", err),
    };
}

#[test]
fn parse_wasm() -> Result<()> {
    let mut file = open_file("mandelbrot.wasm");
    let module = wasm::Module::parse(&mut file)?;
    println!("{:#?}", module);

    Ok(())
}
