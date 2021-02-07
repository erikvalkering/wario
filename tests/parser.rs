use std::fs::File;
use std::io::Read;

type Result<T> = std::result::Result<T, String>;

fn parse_u32(file: &mut File) -> Result<u32> {
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

#[derive(Debug)]
struct Preamble {
    magic: [u8; 4],
    version: [u8; 4],
}

fn parse_preamble(file: &mut File) -> Result<Preamble> {
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
    };

    Ok(Preamble {
        magic: magic,
        version: version,
    })
}

#[derive(Debug)]
enum SectionID {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

#[derive(Debug)]
struct Section {
    id: SectionID,
    size: u32,
    contents: Vec<u8>,
}

fn parse_section(file: &mut File) -> Result<Section> {
    let mut id = [0; 1];
    if let Err(err) = file.read(&mut id) {
        return Err(format!("Unable to read section id: {}", err));
    }

    let id = match id[0] {
        0 => SectionID::Custom,
        1 => SectionID::Type,
        2 => SectionID::Import,
        3 => SectionID::Function,
        4 => SectionID::Table,
        5 => SectionID::Memory,
        6 => SectionID::Global,
        7 => SectionID::Export,
        8 => SectionID::Start,
        9 => SectionID::Element,
        10 => SectionID::Code,
        11 => SectionID::Data,
        _ => return Err(format!("Found unknown section id: {}", id[0])),
    };

    let size = parse_u32(file)?;

    Ok(Section {
        id: id,
        size: size,
        contents: vec![],
    })
}

fn parse_sections(file: &mut File) -> Result<Vec<Section>> {
    Ok(vec![parse_section(file)?])
}

#[derive(Debug)]
struct FuncType;

#[derive(Debug)]
struct Module {
    preamble: Preamble,
    types: Option<Vec<FuncType>>,
}


fn parse_module(file: &mut File) -> Result<Module> {
    let mut module = Module {
        preamble: parse_preamble(file)?,
        types: None,
    };

    for section in parse_sections(file)? {
        println!("{:?}", section);
    }

    Ok(module)
}

#[test]
fn parse_wasm() -> Result<()> {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", "input.wasm"]
        .iter()
        .collect();

    let file = File::open(path);
    let mut file = match file {
        Ok(file) => file,
        Err(err) => return Err(format!("Unable to open file: {}", err)),
    };

    let module = parse_module(&mut file)?;
    println!("{:?}", module);

    Ok(())
}
