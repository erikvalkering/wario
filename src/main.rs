use wario::{ExternFunction, Instruction, Machine, ModuleFunction};

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
            Instruction::Call(1),
            Instruction::Load(0),
            Instruction::Call(0),
            Instruction::Store(0),
        ]),
    ];

    // fn increment(value: i32) {
    //   value + 1
    // }
    let move_player = ModuleFunction {
        param_count: 1,
        code: vec![
            Instruction::LocalGet(0),
            Instruction::Const(1),
            Instruction::Add,
        ],
    };

    let display_player = ExternFunction {
        param_count: 1,
        fun: Box::new(|args: &[i32]| {
            println!("{} B-)", " ".repeat(args[0] as usize));
            None
        }),
    };

    let module_functions = vec![move_player];
    let mut extern_functions = vec![display_player];
    let mut locals = vec![];

    let mut machine = Machine::new();
    machine.debugging = false;

    machine.execute(&code, &module_functions, &mut extern_functions, &mut locals);
}
