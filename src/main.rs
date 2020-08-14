struct Machine {
    stack: Vec<i32>,
}

fn main() {
    let code = vec![
        ("const", 1),
        ("const", 2),
    ];

    let mut machine = Machine{
        stack: Vec::new(),
    };

    for instruction in code {
        match instruction {
            ("const", value) => machine.stack.push(value),
            (operator, _) => eprintln!("Unknown instruction: {}", operator),
        }
    }
}
