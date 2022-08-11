use cranelift::{
    frontend::{FunctionBuilder, FunctionBuilderContext},
    prelude::{types, AbiParam, InstBuilder, Value},
};
use cranelift_codegen::Context;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, DataContext, Linkage, Module};
use math_parser::{parse, Atom, Expr, Op};
use std::collections::HashMap;

pub struct Compiler {
    pub builder_ctx: FunctionBuilderContext,
    pub ctx: Context,
    pub data: DataContext,
    pub module: JITModule,
}

impl Default for Compiler {
    fn default() -> Self {
        let builder = JITBuilder::new(default_libcall_names()).unwrap();
        let module = JITModule::new(builder);
        Self {
            builder_ctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data: DataContext::new(),
            module,
        }
    }
}

impl Compiler {
    pub fn compile(
        &mut self,
        input: &str,
        symbol_table: &mut HashMap<String, Value>,
    ) -> Result<*const u8, String> {
        let ast = match parse(input) {
            Ok(val) => val,
            Err(err) => panic!("{}", err),
        };
        self.translate(ast, symbol_table);
        let id = self
            .module
            .declare_function("main", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())
            .unwrap();
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| format!("{}", e))
            .unwrap();
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();
        let code = self.module.get_finalized_function(id);
        Ok(code)
    }
    fn translate(&mut self, expr: Expr, symbol_table: &mut HashMap<String, Value>) {
        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::F64));
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);
        let mut trans = Translator {
            builder,
            module: &mut self.module,
            var_index: 0,
        };
        let val = trans.translate_expr(expr, symbol_table);
        trans.builder.ins().return_(&[val]);
        trans.builder.finalize();
    }
}
pub struct Translator<'scope> {
    pub builder: FunctionBuilder<'scope>,
    pub module: &'scope mut JITModule,
    pub var_index: usize,
}

impl<'scope> Translator<'scope> {
    fn translate_atom(
        &mut self,
        atom: Atom,
        symbol_table: &mut HashMap<String, Value>,
    ) -> Value {
        match atom {
            Atom::Ident(ref ident) => {
                let var = symbol_table.get(ident).unwrap();
                *var
            }
            Atom::Integer(int) => self.builder.ins().f64const(int),
        }
    }
    fn translate_expr(
        &mut self,
        expr: Expr,
        symbol_table: &mut HashMap<String, Value>,
    ) -> Value {
        match expr {
            Expr::Unary(atom) => self.translate_atom(atom, symbol_table),
            Expr::BinOp {
                lhs: _,
                op: _,
                rhs: _,
            } => self.translate_binop(expr, symbol_table),
        }
    }
    fn translate_binop(
        &mut self,
        expr: Expr,
        symbol_table: &mut HashMap<String, Value>,
    ) -> Value {
        if let Expr::BinOp { lhs, op, rhs } = expr {
            let lhs = self.translate_expr(*lhs, symbol_table);
            let rhs = self.translate_expr(*rhs, symbol_table);
            return match op {
                Op::Add => self.builder.ins().fadd(lhs, rhs),
                Op::Sub => self.builder.ins().fsub(lhs, rhs),
                Op::Mul => self.builder.ins().fmul(lhs, rhs),
                Op::Div => self.builder.ins().fdiv(lhs, rhs),
                Op::Eq => panic!("this is a binding expr not a binary op"),
                _ => unimplemented!(),
            };
        } else {
            unreachable!()
        }
    }
}
