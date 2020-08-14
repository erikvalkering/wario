struct Machine {
    stack: Vec<i32>,
}

enum Instruction {
    Const(i32),
}

fn main() {
    let code: Vec<Instruction> = vec![
        Instruction::Const(1),
        Instruction::Const(2),
    ];

    let mut machine = Machine{
        stack: Vec::new(),
    };

    for instruction in code {
        match instruction {
            Instruction::Const(value) => machine.stack.push(value),
        }
    }
}
