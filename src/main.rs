use wario::{ExternFunction, Instruction, Machine};

fn main() {
    // int i = 0;
    // while (true) {
    //   push(i)
    //   call(print)
    //   i++;
    // }

    let code = vec![
        Instruction::Const(0),
        Instruction::Store(0),
        Instruction::Loop(vec![
            Instruction::Load(0),
            Instruction::Call(0),
            Instruction::Load(0),
            Instruction::Const(1),
            Instruction::Add,
            Instruction::Store(0),
        ]),
    ];

    let display = ExternFunction {
        param_count: 1,
        fun: Box::new(|args: &[i32]| {
            println!("{} B-)", " ".repeat(args[0] as usize));
            None
        }),
    };

    let module_functions = vec![];
    let mut extern_functions = vec![display];
    let mut locals = vec![];

    let mut machine = Machine::new();
    machine.debugging = false;

    machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);
}
