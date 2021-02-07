use std::fs::File;
use std::io::Read;

type Result = std::result::Result<(), String>;

fn parse_preamble(file: &mut File) -> Result {
    let mut data: [u8; 4] = [0; 4];
    if let Err(err) = file.read(&mut data) {
        return Err(format!("Unable to read preamble: {}", err));
    }

    match &data {
        b"\0asm" => Ok(()),
        _ => Err("Invalid preamble".to_owned()),
    }
}

fn parse_sections(file: &mut File) -> Result {
    Ok(())
}

fn parse_module(mut file: &mut File) -> Result {
    parse_preamble(&mut file)?;
    parse_sections(&mut file)?;

    Ok(())
}

#[test]
fn parse_wasm() -> Result {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "input.wasm"]
        .iter()
        .collect();

    let file = File::open(path);
    let mut file = match file {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open file: {}", err)),
    };

    parse_module(&mut file)?;

    Ok(())
}
