use std::fs::File;
use std::io::Read;

type Result = std::result::Result<(), String>;

fn parse_u32(file: &mut File) -> std::result::Result<u32, String> {
    let mut result = 0u32;

    loop {
        let mut buf = [0; 1];
        if let Err(err) = file.read(&mut buf) {
            return Err(format!("Unable to read u32: {}", err));
        }

        result <<= 7;
        result |= buf[0] as u32 & 0x7f;

        if buf[0] & 0x80 == 0 {
            break;
        }
    }

    Ok(result)
}

fn parse_preamble(file: &mut File) -> Result {
    let mut magic = [0; 4];
    if let Err(err) = file.read(&mut magic) {
        return Err(format!("Unable to read preamble: {}", err));
    }

    match &magic {
        b"\0asm" => Ok(()),
        _ => Err("Invalid magic value".to_owned()),
    }?;

    let mut version = [0; 4];
    if let Err(err) = file.read(&mut version) {
        return Err(format!("Unable to read preamble: {}", err));
    }

    match &version {
        [1, 0, 0, 0] => Ok(()),
        _ => Err("Invalid version".to_owned()),
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

    let size = parse_u32(file)?;
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
