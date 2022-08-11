use cranelift::prelude::Value;
use rc_lib::Compiler;
use std::{
    collections::HashMap,
    io::{stdin, stdout, Write},
    mem,
};

fn main() {
    let mut symbols = HashMap::new();
    loop {
        let mut compiler = Compiler::default();
        print!(">>> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        unsafe {
            let output: f64 = run_code(&mut compiler, input, &mut symbols).unwrap();
            println!("{}", output);
        }
    }
}
unsafe fn run_code<O>(
    jit: &mut Compiler,
    code: &str,
    table: &mut HashMap<String, Value>,
) -> Result<O, String> {
    let code_ptr = jit.compile(code, table)?;
    let code_fn = mem::transmute::<_, fn() -> O>(code_ptr);
    Ok(code_fn())
}
