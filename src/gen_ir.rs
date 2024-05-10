///!实现生成Koopa IR
use std::{fs::File, io::Write};

use koopa::front::ast::Return;

use crate::ast::*;
use crate::calc_exp::Eval;
use crate::ds_for_ir::GenerateIrInfo;

use crate::symbol_table::SymbolType::Const;
use crate::symbol_table::SymbolType::Var;
use crate::symbol_table::VarTypeBase;

///用于生成IR的trait
pub trait GenerateIR {
    ///用于记录不同种类单元的返回情况
    type GenerateResult;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> Self::GenerateResult;
}

///用于记录子树中是否已经return
pub enum Returned {
    Yes,
    No,
}

///为CompUnit实现 GenerateIR trait
impl GenerateIR for CompUnit {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        self.func_def.generate(output, info);
    }
}

///为FuncDef实现GenerateIR trait
impl GenerateIR for FuncDef {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        write!(output, "fun @{}", self.ident).unwrap();
        write!(output, "(): ").unwrap();
        self.func_type.generate(output, info);
        write!(output, " ").unwrap();
        write!(output, "{{\n").unwrap();
        write!(output, "%entry:\n").unwrap();
        match self.block.generate(output, info) {
            Returned::Yes => {}
            Returned::No => match self.func_type {
                FuncType::Int => writeln!(output, "  ret 0").unwrap(),
                _ => writeln!(output, "  ret").unwrap(), //TODO以后有void函数再改
            },
        }
        write!(output, "}}\n").unwrap();
    }
}

///为FuncType实现GenerateIR trait
impl GenerateIR for FuncType {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, _: &mut GenerateIrInfo) {
        match self {
            FuncType::Int => write!(output, "i32").unwrap(),
        }
    }
}

///为Block实现GenerateIR trait
impl GenerateIR for Block {
    type GenerateResult = Returned;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> Returned {
        //目前现在block的GenerateIR trait调用新建block
        //注意只有FuncDef和Stmt会推导出Block
        info.push_block();
        for item in &self.items {
            match item.generate(output, info) {
                Returned::Yes => {
                    info.pop_block();
                    return Returned::Yes;
                }
                Returned::No => {}
            }
        }
        info.pop_block();
        Returned::No
    }
}

///为BlockItem实现GenerateIR trait
impl GenerateIR for BlockItem {
    type GenerateResult = Returned;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> Returned {
        match self {
            BlockItem::Decl(decl) => decl.generate(output, info),
            BlockItem::Stmt(stmt) => stmt.generate(output, info),
        }
    }
}

///为Stmt实现GenerateIR trait
impl GenerateIR for Stmt {
    type GenerateResult = Returned;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> Returned {
        match self {
            Stmt::Assign(lval, exp) => {
                //赋值语句，LVal必须是变量
                exp.generate(output, info);
                let exp_id = info.now_id;

                writeln!(
                    output,
                    "  store %{}, @{}",
                    exp_id,
                    info.get_name(&lval.ident).unwrap()
                )
                .unwrap();

                Returned::No
            }
            Stmt::Exp(exp) => {
                match exp {
                    Some(exp) => {
                        exp.generate(output, info);
                    }
                    None => {}
                }
                Returned::No
            }
            Stmt::Block(block) => block.generate(output, info),
            Stmt::RetExp(exp) => {
                match exp {
                    Some(exp) => {
                        exp.generate(output, info);
                        writeln!(output, "  ret %{}", info.now_id).unwrap();
                    }
                    None => {
                        writeln!(output, "  ret").unwrap();
                    }
                }
                Returned::Yes
            }
            Stmt::If(exp, then_stmt, else_stmt) => {
                let exp_id = exp.generate(output, info);
                //当前if else的编号
                info.if_id += 1;
                let now_if_id = info.if_id;
                //if 的条件判断部分
                if else_stmt.is_some() {
                    //else存在
                    writeln!(
                        output,
                        "  br %{}, %if_true_{}, %if_false_{}",
                        exp_id, now_if_id, now_if_id
                    )
                    .unwrap();
                } else {
                    //else不存在
                    writeln!(
                        output,
                        "  br %{}, %if_true_{}, %if_end_{}",
                        exp_id, now_if_id, now_if_id
                    )
                    .unwrap();
                }

                //if 的then部分
                //生成then基础块标号
                writeln!(output, "%if_true_{}:", now_if_id).unwrap();
                //用于记录then和else中是否有return | 好像没用 TODO
                let mut then_else_result = Returned::No;
                match then_stmt.generate(output, info) {
                    Returned::Yes => {
                        //then部分有return，不生成跳转
                        then_else_result = Returned::Yes;
                    }
                    Returned::No => {
                        //then部分没有return，生成跳转
                        writeln!(output, "  jump %if_end_{}", now_if_id).unwrap();
                    }
                }
                if else_stmt.is_some() {
                    //检查else是否存在
                    //if 的else部分
                    let else_stmt = (else_stmt).as_ref().unwrap();
                    //生成else基础块标号
                    writeln!(output, "%if_false_{}:", now_if_id).unwrap();
                    match else_stmt.generate(output, info) {
                        Returned::Yes => {
                            //else部分有return，不生成跳转
                            then_else_result = Returned::Yes;
                        }
                        Returned::No => {
                            //else部分没有return，生成跳转
                            writeln!(output, "  jump %if_end_{}", now_if_id).unwrap();
                        }
                    }
                }
                //生成if结束基础块标号
                writeln!(output, "%if_end_{}:", now_if_id).unwrap();
                //then_else_result
                Returned::No
            }
        }
    }
}

///为Exp实现GenerateIR trait
impl GenerateIR for Exp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            Exp::LOrExp(lor_exp) => lor_exp.generate(output, info),
        }
    }
}

///为UnaryExp实现GenerateIR trait
impl GenerateIR for UnaryExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            UnaryExp::PrimaryExp(primary_exp) => primary_exp.generate(output, info),
            UnaryExp::BinaryOp(op, exp) => {
                let exp_id = exp.generate(output, info);
                match op {
                    UnaryOp::Neg => {
                        info.now_id += 1;
                        writeln!(output, "  %{} = sub 0, %{}", info.now_id, exp_id).unwrap();
                    }
                    UnaryOp::Pos => {}
                    UnaryOp::Not => {
                        info.now_id += 1;
                        writeln!(output, "  %{} = eq 0, %{}", info.now_id, exp_id).unwrap();
                    }
                }
                info.now_id
            }
        }
    }
}

///为PrimaryExp实现GenerateIR trait
impl GenerateIR for PrimaryExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            PrimaryExp::Bexp(exp) => exp.generate(output, info),
            PrimaryExp::Number(num) => {
                info.now_id += 1;
                //这里以后回来改
                writeln!(output, "  %{} = add {}, 0", info.now_id, num).unwrap();
                info.now_id
            }
            PrimaryExp::LVal(lval) => {
                lval.generate(output, info);
                let lval_id = info.now_id;
                info.now_id += 1;
                writeln!(output, "  %{} = add %{}, 0", info.now_id, lval_id).unwrap();
                info.now_id
            }
        }
    }
}

///为AddExp实现GenerateIR trait
impl GenerateIR for AddExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            AddExp::MulExp(mul_exp) => mul_exp.generate(output, info),
            AddExp::BinaryExp(add_exp, op, mul_exp) => {
                let mul_id = mul_exp.generate(output, info);
                let add_id = add_exp.generate(output, info);
                info.now_id += 1;
                write!(output, "  %{} = ", info.now_id).unwrap();
                match op {
                    BinaryAddOp::Add => write!(output, "add").unwrap(),
                    BinaryAddOp::Sub => write!(output, "sub").unwrap(),
                }
                writeln!(output, " %{}, %{}", add_id, mul_id).unwrap();
                info.now_id
            }
        }
    }
}

///为MulExp实现GenerateIR trait
impl GenerateIR for MulExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            MulExp::UnaryExp(unary_exp) => unary_exp.generate(output, info),
            MulExp::BinaryExp(mul_exp, op, unary_exp) => {
                let unary_id = unary_exp.generate(output, info);
                let mul_id = mul_exp.generate(output, info);
                info.now_id += 1;
                write!(output, "  %{} = ", info.now_id).unwrap();
                match op {
                    BinaryMulOp::Mul => write!(output, "mul").unwrap(),
                    BinaryMulOp::Div => write!(output, "div").unwrap(),
                    BinaryMulOp::Mod => write!(output, "mod").unwrap(),
                }
                writeln!(output, " %{}, %{}", mul_id, unary_id).unwrap();
                info.now_id
            }
        }
    }
}

///为RelExp实现GenerateIR trait
impl GenerateIR for RelExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            RelExp::AddExp(add_exp) => add_exp.generate(output, info),
            RelExp::BinaryExp(rel_exp, op, add_exp) => {
                let add_id = add_exp.generate(output, info);
                let rel_id = rel_exp.generate(output, info);
                info.now_id += 1;
                write!(output, "  %{} = ", info.now_id).unwrap();
                match op {
                    BinaryRelOp::Lt => write!(output, "lt").unwrap(),
                    BinaryRelOp::Gt => write!(output, "gt").unwrap(),
                    BinaryRelOp::Le => write!(output, "le").unwrap(),
                    BinaryRelOp::Ge => write!(output, "ge").unwrap(),
                }
                writeln!(output, " %{}, %{}", rel_id, add_id).unwrap();
                info.now_id
            }
        }
    }
}

///为EqExp实现GenerateIR trait
impl GenerateIR for EqExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            EqExp::RelExp(rel_exp) => rel_exp.generate(output, info),
            EqExp::BinaryExp(eq_exp, op, rel_exp) => {
                let rel_id = rel_exp.generate(output, info);
                let eq_id = eq_exp.generate(output, info);
                info.now_id += 1;
                write!(output, "  %{} = ", info.now_id).unwrap();
                match op {
                    BinaryEqOp::Eq => write!(output, "eq").unwrap(),
                    BinaryEqOp::Ne => write!(output, "ne").unwrap(),
                }
                writeln!(output, " %{}, %{}", eq_id, rel_id).unwrap();
                info.now_id
            }
        }
    }
}

///为LAndExp实现GenerateIR trait
///注意应该是实现逻辑and，Koopa IR中的是按位and
impl GenerateIR for LAndExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        match self {
            LAndExp::EqExp(eq_exp) => eq_exp.generate(output, info),
            /*and短路求值逻辑
              @and_result_114 = alloc i32
              store 0, @and_result_114
              %lhs = ...
              %lhs_ne_0_114 = ne %lhs 0
              br %lhs_ne_0_114, %calc_rhs_114, %and_end_114
            %calc_rhs_114:
              %rhs = ...
              %rhs_ne_0_114 = ne %rhs, 0
              store %rhs_ne_0_114, @and_result_114
              jump %and_end_114
            %and_end_114:
              %ans = load @and_result_114
            */
            LAndExp::BinaryExp(land_exp, eq_exp) => {
                info.and_or_id += 1;
                let now_and_or_id = info.and_or_id;
                writeln!(output, "  @and_result_{} = alloc i32", now_and_or_id).unwrap();
                writeln!(output, "  store 0, @and_result_{}", now_and_or_id).unwrap();
                let lhs_id = land_exp.generate(output, info);
                writeln!(output, "  %lhs_ne_0_{} = ne %{}, 0", now_and_or_id, lhs_id).unwrap();
                writeln!(
                    output,
                    "  br %lhs_ne_0_{}, %calc_rhs_{}, %and_end_{}",
                    now_and_or_id, now_and_or_id, now_and_or_id
                )
                .unwrap();
                writeln!(output, "%calc_rhs_{}:", now_and_or_id).unwrap();
                let rhs_id = eq_exp.generate(output, info);
                writeln!(output, "  %rhs_ne_0_{} = ne %{}, 0", now_and_or_id, rhs_id).unwrap();
                writeln!(
                    output,
                    "  store %rhs_ne_0_{}, @and_result_{}",
                    now_and_or_id, now_and_or_id
                )
                .unwrap();
                writeln!(output, "  jump %and_end_{}", now_and_or_id).unwrap();
                writeln!(output, "%and_end_{}:", now_and_or_id).unwrap();
                info.now_id += 1;
                writeln!(
                    output,
                    "  %{} = load @and_result_{}",
                    info.now_id, now_and_or_id
                )
                .unwrap();
                info.now_id
                /*
                    //land != 0
                    info.now_id += 1;
                    let land_not_0 = info.now_id;
                    writeln!(output, "  %{} = ne 0, %{}", land_not_0, land_id).unwrap();

                    //(eq != 0) & (land != 0)
                    info.now_id += 1;
                    writeln!(
                        output,
                        "  %{} = and %{}, %{}",
                        info.now_id, land_not_0, eq_not_0
                    )
                    .unwrap();
                */
            }
        }
    }
}

///为LOrExp实现GenerateIR trait
impl GenerateIR for LOrExp {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return info.now_id;
        }
        /*or短路求值逻辑
          @or_result_114 = alloc i32
          store 1, @or_result_114
          %lhs = ...
          %lhs_eq_0_114 = eq %lhs 0
          br %lhs_eq_0_114, %calc_rhs_114, %or_end_114
        %calc_rhs_114:
          %rhs = ...
          %rhs_ne_0_114 = ne %rhs, 0
          store %rhs_ne_0_114, @or_result_114
          jump %or_end_114
        %or_end_114:
          %ans = load @or_result_114
        */
        match self {
            LOrExp::LAndExp(land_exp) => land_exp.generate(output, info),
            LOrExp::BinaryExp(lor_exp, land_exp) => {
                info.and_or_id += 1;
                let now_and_or_id = info.and_or_id;
                writeln!(output, "  @or_result_{} = alloc i32", now_and_or_id).unwrap();
                writeln!(output, "  store 1, @or_result_{}", now_and_or_id).unwrap();
                let lhs_id = lor_exp.generate(output, info);
                writeln!(output, "  %lhs_eq_0_{} = eq %{}, 0", now_and_or_id, lhs_id).unwrap();
                writeln!(
                    output,
                    "  br %lhs_eq_0_{}, %calc_rhs_{}, %or_end_{}",
                    now_and_or_id, now_and_or_id, now_and_or_id
                )
                .unwrap();
                writeln!(output, "%calc_rhs_{}:", now_and_or_id).unwrap();
                let rhs_id = land_exp.generate(output, info);
                writeln!(output, "  %rhs_ne_0_{} = ne %{}, 0", now_and_or_id, rhs_id).unwrap();
                writeln!(
                    output,
                    "  store %rhs_ne_0_{}, @or_result_{}",
                    now_and_or_id, now_and_or_id
                )
                .unwrap();
                writeln!(output, "  jump %or_end_{}", now_and_or_id).unwrap();
                writeln!(output, "%or_end_{}:", now_and_or_id).unwrap();
                info.now_id += 1;
                writeln!(
                    output,
                    "  %{} = load @or_result_{}",
                    info.now_id, now_and_or_id
                )
                .unwrap();

                info.now_id
                /*
                    //land != 0
                    info.now_id += 1;
                    let land_not_0 = info.now_id;
                    writeln!(output, "  %{} = ne 0, %{}", land_not_0, land_id).unwrap();

                    //lor != 0
                    info.now_id += 1;
                    let lor_not_0 = info.now_id;
                    writeln!(output, "  %{} = ne 0, %{}", lor_not_0, lor_id).unwrap();

                    //(lor != 0) | (land != 0)
                    info.now_id += 1;
                    writeln!(
                        output,
                        "  %{} = or %{}, %{}",
                        info.now_id, lor_not_0, land_not_0
                    )
                    .unwrap();
                */
            }
        }
    }
}

///为Decl实现GenerateIR trait
impl GenerateIR for Decl {
    type GenerateResult = Returned;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> Returned {
        match self {
            Decl::ConstDecl(const_decl) => const_decl.generate(output, info),
            Decl::VarDecl(var_decl) => var_decl.generate(output, info),
        }
        Returned::No
    }
}

///为ConstDecl实现GenerateIR trait
impl GenerateIR for ConstDecl {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        match self {
            ConstDecl::ConstDeclS(btype, const_def_s) => {
                for const_def in const_def_s {
                    const_def.generate(output, info);
                }
            }
        }
    }
}

///为ConstDef实现GenerateIR trait
impl GenerateIR for ConstDef {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        match self.const_init_val.eval(info) {
            Some(val) => {
                info.insert_symbol(self.ident.clone(), Const(val));
            }
            None => panic!("detected Var in ConstDef when evaluating"),
        }
    }
}

///为VarDecl实现GenerateIR trait
impl GenerateIR for VarDecl {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        match self {
            VarDecl::VarDeclS(btype, var_def_s) => {
                for var_def in var_def_s {
                    var_def.generate(output, info);
                }
            }
        }
    }
}

///为VarDef实现GenerateIR trait
impl GenerateIR for VarDef {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        match self {
            VarDef::NoInit(ident) => {
                info.insert_symbol(ident.clone(), Var(VarTypeBase::new()));
                writeln!(output, "  @{} = alloc i32", info.get_name(ident).unwrap()).unwrap();
            }
            VarDef::Init(ident, init_val) => {
                info.insert_symbol(ident.clone(), Var(VarTypeBase::new()));
                writeln!(output, "  @{} = alloc i32", info.get_name(ident).unwrap()).unwrap();

                init_val.generate(output, info);
                let init_val_id = info.now_id;

                writeln!(
                    output,
                    "  store %{}, @{}",
                    init_val_id,
                    info.get_name(&ident).unwrap()
                )
                .unwrap();
            }
        }
    }
}

///为InitVal实现GenerateIR trait
impl GenerateIR for InitVal {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        match self {
            InitVal::Exp(exp) => {
                exp.generate(output, info);
            }
        }
    }
}

///为LVal实现GenerateIR trait
///作用是取出LVal对应的变量的值，存入info.now_id + 1中
impl GenerateIR for LVal {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        let eval_result = self.eval(info);
        if eval_result.is_some() {
            info.now_id += 1;
            writeln!(
                output,
                "  %{} = add {}, 0",
                info.now_id,
                eval_result.unwrap()
            )
            .unwrap();
            return;
        }
        let x = info.search_symbol(&self.ident).unwrap();
        info.now_id += 1;
        match x.content {
            Var(_) => {
                writeln!(
                    output,
                    "  %{} = load @{}",
                    info.now_id,
                    info.get_name(&self.ident).unwrap()
                )
                .unwrap();
            }
            Const(val) => {
                writeln!(output, "  %{} = add {}, 0", info.now_id, val).unwrap();
            }
        }
    }
}
