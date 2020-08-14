struct Machine {
    stack: Vec<i32>,
}

impl Machine {
    fn new() -> Self {
        Machine{
            stack: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum Instruction {
    Const(i32),
    Add,
}

impl Machine {
    fn interpret(self: &mut Self, code: Vec<Instruction>) {
        for instruction in code {
            print!("> {:?}", instruction);

            match instruction {
                Instruction::Const(value) => self.stack.push(value),
                Instruction::Add => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                }
            }

            println!(" => {:?}", self.stack);
        }
    }
}

fn main() {
    let code: Vec<Instruction> = vec![
        Instruction::Const(1),
        Instruction::Const(2),
        Instruction::Add,
    ];

    let mut machine = Machine::new();
    machine.interpret(code);
}
