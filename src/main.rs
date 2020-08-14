struct Machine {
    stack: Vec<i32>,
}

enum Instruction {
    Const(i32),
    Add,
}

fn main() {
    let code: Vec<Instruction> = vec![
        Instruction::Const(1),
        Instruction::Const(2),
        Instruction::Add,
    ];

    let mut machine = Machine{
        stack: Vec::new(),
    };

    for instruction in code {
        match instruction {
            Instruction::Const(value) => machine.stack.push(value),
            Instruction::Add => {
                let a = machine.stack.pop().unwrap();
                let b = machine.stack.pop().unwrap();
                machine.stack.push(a + b);
            }
        }
    }

    println!("Result: {}", machine.stack.pop().unwrap());
}
