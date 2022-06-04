use cranelift::{
    codegen::Context,
    frontend::{FunctionBuilder, FunctionBuilderContext, Variable},
    prelude::{EntityRef, InstBuilder, Value, AbiParam, types, Block},
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, DataContext, Linkage, Module};
use math_parser::{Atom, Expr, Op};
use std::{collections::HashMap};

#[allow(dead_code)]
pub struct Compiler {
    pub ctx: Context,
    pub data: DataContext,
    pub module: JITModule,
    pub builder_ctx: FunctionBuilderContext
}

impl Default for Compiler {
    fn default() -> Self {
        let builder = JITBuilder::new(default_libcall_names()).unwrap();
        let module = JITModule::new(builder);
        let mut ctx = module.make_context();
        Self {
            ctx,
            data: DataContext::new(),
            module,
            builder_ctx: FunctionBuilderContext::new()
        }
    }
}

impl Compiler {
    pub fn compile(
        &mut self,
        input: Expr,
        mut func_builder: FunctionBuilder,
    ) -> Result<*const u8, String> {
        let entry = func_builder.create_block();
        func_builder.append_block_params_for_function_params(entry);
        func_builder.switch_to_block(entry);
        func_builder.seal_block(entry);
        let id = self
            .module
            .declare_function("calc_main", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();
        let code = self.module.get_finalized_function(id);
        Ok(code)
    }
    pub fn translate(
        &mut self,
        params: Vec<String>,
        the_return: String,
        stmts: Expr,
    ) -> Result<(), String> {
        // Our toy language currently only supports I64 values, though Cranelift
        // supports other types.
        let int = self.module.target_config().pointer_type();

        for _p in &params {
            self.ctx.func.signature.params.push(AbiParam::new(int));
        }

        // Our toy language currently only supports one return value, though
        // Cranelift is designed to support more.
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        // Create the builder to build a function.
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_ctx);

        // Create the entry block, to start emitting code in.
        let entry_block = builder.create_block();

        // Since this is the entry block, add block parameters corresponding to
        // the function's parameters.
        //
        // TODO: Streamline the API here.
        builder.append_block_params_for_function_params(entry_block);

        // Tell the builder to emit code in this block.
        builder.switch_to_block(entry_block);

        // And, tell the builder that this block will have no further
        // predecessors. Since it's the entry block, it won't have any
        // predecessors.
        builder.seal_block(entry_block);

        // The toy language allows variables to be declared implicitly.
        // Walk the AST and declare all implicitly-declared variables.
        let variables =
            declare_variables(int, &mut builder, &params, &the_return, &stmts, entry_block);

        // Now translate the statements of the function body.
        let mut trans = Translator {
            var_index: 0,
            builder,
            symbol_table: variables,
            module: &mut self.module,
        };
        for expr in stmts {
            trans.compile_expr(expr);
        }

        // Set up the return variable of the function. Above, we declared a
        // variable to hold the return value. Here, we just do a use of that
        // variable.
        let return_variable = trans.symbol_table.get(&the_return).unwrap();
        let return_value = trans.builder.use_var(*return_variable);

        // Emit the return instruction.
        trans.builder.ins().return_(&[return_value]);

        // Tell the builder we're done with this function.
        trans.builder.finalize();
        Ok(())
    }
}

pub struct Translator<'scope> {
    pub builder: FunctionBuilder<'scope>,
    pub symbol_table: HashMap<String, Variable>,
    pub var_index: usize,
    pub module: &'scope mut JITModule
}

impl<'scope> Translator<'scope> {
    pub fn compile_expr(&self, expr: Expr) -> Value {
        match expr {
            Expr::Unary(atom) => self.compile_atom(atom),
            Expr::BinOp {
                lhs: _,
                ref op,
                rhs: _,
            } if matches!(op, &Op::Eq) => self.compile_bindig(expr),
            Expr::BinOp {
                lhs: _,
                op: _,
                rhs: _,
            } => todo!(),
        }
    }
    fn compile_atom(&self, atom: Atom) -> Value {
        match atom {
            Atom::Ident(ref name) => match self.symbol_table.get(name) {
                Some(var) => self.builder.use_var(*var),
                None => self.builder.ins().f64const(f64::default()),
            },
            Atom::Integer(int) => self.builder.ins().f64const(int),
        }
    }
    fn compile_bindig(&self, expr: Expr) -> Value {
        if let Expr::BinOp { lhs, op, rhs } = expr {
            if op != Op::Eq {
                panic!("not a binding expression")
            }
            let rhs = self.compile_expr(*rhs);
            let var = Variable::new(self.var_index);
            self.var_index += 1;
            self.builder.def_var(var, rhs);
            let mut symbol_table = self.symbol_table;
            if let Expr::Unary(Atom::Ident(lhs)) = *lhs {
                if symbol_table.contains_key(lhs.as_str()) {
                    symbol_table.remove(lhs.as_str());
                }
                symbol_table.insert(lhs, var);
            }
            return rhs;
        } else {
            panic!("not a binding expression")
        }
    }
}

unsafe impl Send for Compiler {}
unsafe impl Sync for Compiler {}
fn declare_variables(
    int: types::Type,
    builder: &mut FunctionBuilder,
    params: &[String],
    the_return: &str,
    stmts: &[Expr],
    entry_block: Block,
) -> HashMap<String, Variable> {
    let mut variables = HashMap::new();
    let mut index = 0;

    for (i, name) in params.iter().enumerate() {
        // TODO: cranelift_frontend should really have an API to make it easy to set
        // up param variables.
        let val = builder.block_params(entry_block)[i];
        let var = declare_variable(int, builder, &mut variables, &mut index, name);
        builder.def_var(var, val);
    }
    let zero = builder.ins().iconst(int, 0);
    let return_variable = declare_variable(int, builder, &mut variables, &mut index, the_return);
    builder.def_var(return_variable, zero);

    variables
}
fn declare_variable(
    int: types::Type,
    builder: &mut FunctionBuilder,
    variables: &mut HashMap<String, Variable>,
    index: &mut usize,
    name: &str,
) -> Variable {
    let var = Variable::new(*index);
    if !variables.contains_key(name) {
        variables.insert(name.into(), var);
        builder.declare_var(var, int);
        *index += 1;
    }
    var
}
