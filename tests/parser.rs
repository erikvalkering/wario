use byteorder::{ByteOrder, LittleEndian};
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

fn parse_section(file: &mut File) -> Result {
    let mut id = [0; 1];
    if let Err(err) = file.read(&mut id) {
        return Err(format!("Unable to read section id: {}", err));
    }

    match id[0] {
        0 => println!("Custom section"),
        1 => println!("Type section"),
        2 => println!("Import section"),
        3 => println!("Function section"),
        4 => println!("Table section"),
        5 => println!("Memory section"),
        6 => println!("Global section"),
        7 => println!("Export section"),
        8 => println!("Start section"),
        9 => println!("Element section"),
        10 => println!("Code section"),
        11 => println!("Data section"),
        _ => return Err(format!("Found unknown section id: {}", id[0])),
    }

    let mut buf = [0; 4];
    if let Err(err) = file.read(&mut buf) {
        return Err(format!("Unable to read section id: {}", err));
    }

    let size = LittleEndian::read_u32(&buf);
    println!("Size: {}", size);

    Ok(())
}

fn parse_sections(file: &mut File) -> Result {
    parse_section(file)?;

    Ok(())
}

fn parse_module(file: &mut File) -> Result {
    parse_preamble(file)?;
    parse_sections(file)?;

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
