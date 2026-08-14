use std::fs::File;

use wario::parser::Result;
use wario::wasm;
use wario::vm::Machine;

fn open_file(filename: &str) -> File {
    use std::path::PathBuf;
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "tests", filename]
        .iter()
        .collect();

    match File::open(path) {
        Ok(file) => file,
        Err(err) => panic!("Unable to open file: {}", err),
    }
}

#[test]
fn parse_wasm() -> Result<()> {
    let mut file = open_file("mandelbrot.wasm");
    let module = wasm::Module::parse(&mut file)?;
    println!("{:#?}", module);

    Ok(())
}

#[test]
fn run_wasm() -> Result<()> {
    let mut file = open_file("mandelbrot.wasm");
    let module = wasm::Module::parse(&mut file)?;

    // let desc = module.exports[0].descriptor;
    // let idx = match desc {
    //     ExportDescriptor::Func(idx) => idx,
    //     _ => panic!("Invalid type: {:?}", desc),
    // };

    // let func = wasm::Func {
    //     ftype: module.types[idx.0],
    //     code: module.codes[idx.0],
    // };

    println!("func: {:#?}", module.func(wasm::FuncIdx(2)));
    // println!("funcs: {:#?}", module.funcs.len());
    // println!("codes: {:#?}", module.codes.len());
    // println!("funcs: {:#?}", module.funcs.len());
    // println!("Exports: {:#?}", module.exports);
    // println!("Function: {:#?}", func);

    let mut machine = Machine::new();
    machine.debugging = true;

    let code = &module.func(wasm::FuncIdx(0)).code;

    let module_functions = vec![];
    let mut extern_functions = vec![];
    let mut locals = vec![];

    machine.invoke(&code.body, &module_functions, &mut extern_functions, &mut locals);

    Err("asdf".to_owned())
}
