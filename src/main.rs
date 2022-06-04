use cranelift::frontend::FunctionBuilder;
use math_parser::parse;
use rc_lib::Compiler;
use std::{
    io::{stdin, stdout, Write},
    mem,
};

fn main() {
    let mut jit = Compiler::default();
    print!(">>> ");
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    unsafe {
        let builder = FunctionBuilder::new(&mut jit.ctx.func, &mut jit.builder_ctx);
        let code = match parse(input.as_str()) {
            Ok(ast) => ast,
            Err(err) => panic!("{}", err),
        };
        let code_ptr = jit.compile(code, builder).unwrap();
        let code_fn = transmute::<f64>(code_ptr);
        let output = code_fn();
        println!("{}", output);
    }
}
unsafe fn transmute<O>(ptr: *const u8) -> fn() -> O {
    mem::transmute::<*const u8, fn() -> O>(ptr)
}
