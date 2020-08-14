struct Machine {
    stack: Vec<i32>,
}

#[derive(Debug)]
enum Instruction {
    Const(i32),
    Add,
}

fn interpret(code: Vec<Instruction>) {
    let mut machine = Machine{
        stack: Vec::new(),
    };

    for instruction in code {
        print!("> {:?}", instruction);

        match instruction {
            Instruction::Const(value) => machine.stack.push(value),
            Instruction::Add => {
                let a = machine.stack.pop().unwrap();
                let b = machine.stack.pop().unwrap();
                machine.stack.push(a + b);
            }
        }

        println!(" => {:?}", machine.stack);
    }
}

fn main() {
    let code: Vec<Instruction> = vec![
        Instruction::Const(1),
        Instruction::Const(2),
        Instruction::Add,
    ];

    interpret(code);
}
