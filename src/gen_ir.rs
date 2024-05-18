use std::fmt::write;
use std::mem::Discriminant;
use std::result;
///!实现生成Koopa IR
use std::{fs::File, io::Write};

use koopa::front::ast::Return;

use crate::ast::*;
use crate::calc_exp::Eval;
use crate::ds_for_ir::GenerateIrInfo;

use crate::array_solve::GenDefDim;
use crate::array_solve::GlobalArrayInit;
use crate::array_solve::LocalArrayInit;
use crate::symbol_table::ArrayInfoBase;
use crate::symbol_table::FuncInfoBase;
use crate::symbol_table::SymbolInfo;
use crate::symbol_table::SymbolInfo::Array;
use crate::symbol_table::SymbolInfo::Const;
use crate::symbol_table::SymbolInfo::Func;
use crate::symbol_table::SymbolInfo::Var;
use crate::symbol_table::VarInfoBase;
///用于生成IR的trait
pub trait GenerateIR {
    ///用于记录不同种类单元的返回情况
    type GenerateResult;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> Self::GenerateResult;
}

///用于记录子树中是否已经return/jump/br
pub enum Returned {
    Yes,
    No,
}

///为CompUnit实现 GenerateIR trait
impl GenerateIR for CompUnit {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        symbol_table_debug!(
            "程序开始,符号表和block表分别为{:#?}\n{:#?}",
            info.tables,
            info.block_id
        );
        //先加入所有SysY库函数的声明
        let lib_func_def = "decl @getint(): i32\n".to_string()
            + "decl @getch(): i32\n"
            + "decl @getarray(*i32): i32\n"
            + "decl @putint(i32)\n"
            + "decl @putch(i32)\n"
            + "decl @putarray(i32, *i32)\n"
            + "decl @starttime()\n"
            + "decl @stoptime()\n";
        writeln!(output, "{}", lib_func_def).unwrap();
        //然后把它们插入到全局符号表
        info.insert_global_symbol("getint".to_string(), Func(FuncInfoBase::new(FuncType::Int)));
        info.insert_global_symbol("getch".to_string(), Func(FuncInfoBase::new(FuncType::Int)));
        info.insert_global_symbol(
            "getarray".to_string(),
            Func(FuncInfoBase::new(FuncType::Int)),
        );
        info.insert_global_symbol(
            "putint".to_string(),
            Func(FuncInfoBase::new(FuncType::Void)),
        );
        info.insert_global_symbol("putch".to_string(), Func(FuncInfoBase::new(FuncType::Void)));
        info.insert_global_symbol(
            "putarray".to_string(),
            Func(FuncInfoBase::new(FuncType::Void)),
        );
        info.insert_global_symbol(
            "starttime".to_string(),
            Func(FuncInfoBase::new(FuncType::Void)),
        );
        info.insert_global_symbol(
            "stoptime".to_string(),
            Func(FuncInfoBase::new(FuncType::Void)),
        );

        for item in &self.item {
            item.generate(output, info);
        }
    }
}

///为CompItem实现GenerateIR trait
impl GenerateIR for CompItem {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        match self {
            CompItem::FuncDef(func_def) => {
                func_def.generate(output, info);
            }
            CompItem::Decl(decl) => {
                decl.generate(output, info);
            }
        }
    }
}

///为FuncDef实现GenerateIR trait
impl GenerateIR for FuncDef {
    type GenerateResult = ();
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) {
        //每个函数层都是一个新的作用域层，具体来说是 {func {block} }
        //这样我们可以保证形参的作用域大于block中的任何变量
        info.push_block();
        info.insert_global_symbol(self.ident.clone(), Func(FuncInfoBase::new(self.func_type)));
        write!(output, "fun @{}", info.get_name(&self.ident)).unwrap();
        write!(output, "(").unwrap();
        for (i, func_fparam) in self.func_fparams.iter().enumerate() {
            match &func_fparam.dims {
                None => {
                    if i != 0 {
                        write!(output, ", ").unwrap();
                    }

                    info.insert_symbol(func_fparam.ident.clone(), Var(VarInfoBase::new()));

                    write!(output, "%{}: ", info.get_name(&func_fparam.ident)).unwrap();
                    match &func_fparam.btype {
                        BType::Int => write!(output, "i32").unwrap(),
                    }
                }
                Some(dims) => {
                    panic!("TODO");
                }
            }
        }
        write!(output, ")").unwrap();

        match self.func_type {
            FuncType::Int => write!(output, ": i32").unwrap(),
            FuncType::Void => {}
        }
        write!(output, " ").unwrap();
        write!(output, "{{\n").unwrap();
        write!(output, "%entry:\n").unwrap();

        //先将形参复制为临时变量，便于后续生成目标代码
        for func_fparam in &self.func_fparams {
            match &func_fparam.dims {
                None => {
                    let param_name = info.get_name(&func_fparam.ident);
                    writeln!(output, "  @{} = alloc i32", param_name).unwrap();
                    writeln!(output, "  store %{}, @{}", param_name, param_name).unwrap();
                    //这里注意到底用%还是@取决于LVal生成的load是啥
                }
                Some(dims) => {
                    panic!("TODO");
                }
            }
        }

        match self.block.generate(output, info) {
            Returned::Yes => {}
            Returned::No => match self.func_type {
                FuncType::Int => writeln!(output, "  ret 0").unwrap(),
                FuncType::Void => writeln!(output, "  ret").unwrap(),
            },
        }
        write!(output, "}}\n").unwrap();
        //记得删除函数层block
        info.pop_block();
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
                //赋值语句
                let exp_id = exp.generate(output, info); //计算右端exp的值

                if lval.dims.is_empty() {
                    //LVal是变量

                    writeln!(
                        output,
                        "  store %{}, @{}",
                        exp_id,
                        info.get_name(&lval.ident)
                    )
                    .unwrap();
                } else {
                    //LVal是数组
                    //先计算目标位置的地址
                    let array_ptr_id = lval.generate(output, info);
                    writeln!(output, "  store %{}, %{}", exp_id, array_ptr_id).unwrap();
                }
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
            Stmt::While(exp, stmt) => {
                //生成while基础块
                info.push_while();
                let now_while_id = info.while_id;
                writeln!(output, "  jump %while_begin_{}", now_while_id).unwrap();
                writeln!(output, "%while_begin_{}:", now_while_id).unwrap();
                let exp_id = exp.generate(output, info);
                writeln!(
                    output,
                    "  br %{}, %while_body_{}, %while_end_{}",
                    exp_id, now_while_id, now_while_id
                )
                .unwrap();
                //生成while循环体基础块标号
                writeln!(output, "%while_body_{}:", now_while_id).unwrap();
                match stmt.generate(output, info) {
                    Returned::Yes => {
                        //循环体有return，不生成跳转
                    }
                    Returned::No => {
                        //循环体没有return，生成跳转
                        writeln!(output, "  jump %while_begin_{}", now_while_id).unwrap();
                    }
                }
                writeln!(output, "%while_end_{}:", now_while_id).unwrap();
                //删除while基础块
                info.pop_while();
                Returned::No
            }
            Stmt::Break => {
                writeln!(
                    output,
                    "  jump %while_end_{}",
                    info.while_history
                        .last()
                        .expect("没有在while中使用break!!!")
                )
                .unwrap();
                Returned::Yes
            }
            Stmt::Continue => {
                writeln!(
                    output,
                    "  jump %while_begin_{}",
                    info.while_history
                        .last()
                        .expect("没有在while中使用continue!!!")
                )
                .unwrap();
                Returned::Yes
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
            UnaryExp::Call(ident, exps) => {
                //计算每个形参表达式
                let mut args = vec![];
                for exp in exps {
                    args.push(exp.generate(output, info));
                    //TODO????
                }
                let x = info.search_symbol(ident).unwrap();
                match x.content {
                    Func(func_info) => match func_info.ret_type {
                        FuncType::Void => {
                            write!(output, "  call @{}", info.get_name(&ident)).unwrap();
                        }
                        FuncType::Int => {
                            info.now_id += 1;
                            write!(
                                output,
                                "  %{} = call @{}",
                                info.now_id,
                                info.get_name(&ident)
                            )
                            .unwrap();
                        }
                    },
                    _ => panic!("尝试调用非函数"),
                }
                write!(output, "(").unwrap();
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        write!(output, ", ").unwrap();
                    }
                    write!(output, "%{}", arg).unwrap();
                }
                writeln!(output, ")").unwrap();
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
                if lval.dims.is_empty() {
                    //如果是变量
                    let lval_id = lval.generate(output, info);
                    info.now_id += 1;
                    writeln!(output, "  %{} = add %{}, 0", info.now_id, lval_id).unwrap();
                } else {
                    //如果是数组
                    let lval_id = lval.generate(output, info);
                    info.now_id += 1;
                    writeln!(output, "  %{} = load %{}", info.now_id, lval_id).unwrap();
                }
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
        if self.dims.is_empty() {
            //如果dims为空，则为常量定义
            match self.const_init_val.eval(info) {
                Some(val) => {
                    info.insert_symbol(self.ident.clone(), Const(val));
                }
                None => panic!("detected Var in ConstDef when evaluating"),
            }
        } else {
            //如果dims不为空，则为常量数组定义，数组名用@开头

            //生成维度声明并加入符号表
            let real_dims = self.gen_def_dim(output, info);

            //填充初始化内容表
            let mut result: Vec<i32> = vec![];
            self.const_init_val
                .global_array_init(output, info, &real_dims, &mut result);

            //为全局生成初始化内容，为局部生成初始化指令
            match info.is_global_symbol(&self.ident) {
                true => {
                    if result.len() == 0 {
                        panic!("可能由数组初值为{{}}引起");
                    } else {
                        write!(output, ", ").unwrap();
                        gen_global_array_ir(output, &real_dims, &result, 0);
                    }

                    writeln!(output, "").unwrap(); //换行
                }
                false => {
                    //局部常量数组初始化
                    write!(output, "\n").unwrap(); //换行

                    if result.len() == 0 {
                        panic!("可能由数组初值为{{}}引起");
                    } else {
                        gen_local_const_array_ir(
                            output,
                            info,
                            &real_dims,
                            &result,
                            0,
                            "@".to_string() + &info.get_name(&self.ident),
                        );
                    }
                }
            }
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
        if self.dims.is_empty() {
            //如果是变量
            match &self.init_val {
                None => {
                    //没有初值
                    if self.dims.is_empty() {
                        //纯变量，非数组
                        info.insert_symbol(self.ident.clone(), Var(VarInfoBase::new()));
                        match info.is_global_symbol(&self.ident) {
                            true => writeln!(
                                output,
                                "global @{} = alloc i32, zeroinit",
                                info.get_name(&self.ident)
                            )
                            .unwrap(),
                            false => {
                                writeln!(output, "  @{} = alloc i32", info.get_name(&self.ident))
                                    .unwrap()
                            }
                        }
                    }
                }
                Some(init_val) => {
                    //纯变量，非数组
                    info.insert_symbol(self.ident.clone(), Var(VarInfoBase::new()));
                    match info.is_global_symbol(&self.ident) {
                        true => writeln!(
                            output,
                            "global @{} = alloc i32, {}",
                            info.get_name(&self.ident),
                            init_val.eval(info).unwrap()
                        )
                        .unwrap(),
                        false => {
                            writeln!(output, "  @{} = alloc i32", info.get_name(&self.ident))
                                .unwrap();

                            init_val.generate(output, info);
                            let init_val_id = info.now_id;

                            writeln!(
                                output,
                                "  store %{}, @{}",
                                init_val_id,
                                info.get_name(&self.ident)
                            )
                            .unwrap();
                        }
                    }
                }
            }
        } else {
            //数组

            //生成维度声明并加入符号表
            let real_dims = self.gen_def_dim(output, info);

            match &self.init_val {
                None => {
                    //没有初值
                    //TODO : 未初始化变量数组声明
                    match info.is_global_symbol(&self.ident) {
                        true => {
                            //全局未初始化自动初始化为0
                            writeln!(output, ", zeroinit").unwrap();
                        }
                        false => {
                            //局部变量数组未初始化不用管!
                            /*
                            SysY规范：未显式初始化的局部变量, 其值是不确定的;
                            而未显式初始化的全局变量, 其 (元素) 值均被初始化为 0.
                            */
                        }
                    }
                }
                Some(init_val) => {
                    //有初值的变量数组初始化

                    match info.is_global_symbol(&self.ident) {
                        true => {
                            //全局有初值变量数组
                            let mut result: Vec<i32> = vec![];
                            init_val.global_array_init(output, info, &real_dims, &mut result);
                            if result.len() == 0 {
                                //初始值是{}，初始化为0
                                panic!("可能由数组初值为{{}}引起");
                            } else {
                                write!(output, ", ").unwrap();
                                gen_global_array_ir(output, &real_dims, &result, 0);
                            }

                            writeln!(output, "").unwrap(); //换行
                        }
                        false => {
                            //局部有初值变量数组

                            write!(output, "\n").unwrap(); //换行

                            let mut result: Vec<i32> = vec![];
                            init_val.local_array_init(output, info, &real_dims, &mut result);
                            if result.len() == 0 {
                                //局部变量数组初值是{}，咋办?不管？ TODO
                                panic!("可能由数组初值为{{}}引起");
                            } else {
                                //TODO
                                gen_local_var_array_ir(
                                    output,
                                    info,
                                    &real_dims,
                                    &result,
                                    0,
                                    "@".to_string() + &info.get_name(&self.ident),
                                );
                            }
                        }
                    }
                }
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
            _ => {
                //TODO : 数组的初始化列表
            }
        }
    }
}

///为LVal实现GenerateIR trait
///作用是取出LVal对应的变量的值，存入返回值中
///或者是将数组对应位置的指针值放在返回值中
impl GenerateIR for LVal {
    type GenerateResult = i32;
    fn generate(&self, output: &mut File, info: &mut GenerateIrInfo) -> i32 {
        let x = info.search_symbol(&self.ident).unwrap();
        match x.content {
            Var(_) => {
                //LVal是变量
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
                } //如果可以编译期间计算，直接返回计算结果

                info.now_id += 1;
                writeln!(
                    output,
                    "  %{} = load @{}",
                    info.now_id,
                    info.get_name(&self.ident)
                )
                .unwrap();
                info.now_id
            }
            Const(val) => {
                //LVal是常量
                info.now_id += 1;
                writeln!(output, "  %{} = add {}, 0", info.now_id, val).unwrap();
                info.now_id
            }
            Array(array_info) => {
                //LVal是数组
                //将数组的指针存入返回值中（可能部分解引用）
                let mut last_base_string = "@".to_string() + &info.get_name(&self.ident);
                for dim in &self.dims {
                    let dim_id = dim.generate(output, info);
                    info.now_id += 1;
                    writeln!(
                        output,
                        "  %{} = getelemptr {}, %{}",
                        info.now_id, last_base_string, dim_id
                    )
                    .unwrap();
                    last_base_string = "%".to_owned() + info.now_id.to_string().as_str();
                }
                if array_info.dims.len() > self.dims.len() {
                    //部分解引用
                    info.now_id += 1;
                    writeln!(
                        output,
                        "  %{} = getelemptr {}, 0",
                        info.now_id, last_base_string
                    )
                    .unwrap();
                }
                info.now_id
            }
            Func(_) => {
                panic!("尝试查询函数的值");
            }
        }
    }
}

///为全局数组生成代码
fn gen_global_array_ir(output: &mut File, dims: &[i32], result: &Vec<i32>, now_pos: i32) {
    if dims.is_empty() {
        //到达叶子
        write!(output, "{}", result[now_pos as usize]).unwrap();
    } else {
        //未到达叶子
        //计算增量，是dims[1..]的乘积
        let mut delta: i32 = 1;
        for dim in &dims[1..] {
            delta *= dim;
        }
        write!(output, "{{").unwrap();

        for i in 0..dims[0] {
            if i != 0 {
                write!(output, ", ").unwrap();
            }
            gen_global_array_ir(output, &dims[1..], result, now_pos + delta * i);
        }

        write!(output, "}}").unwrap();
    }
}

///为局部变量数组生成代码
fn gen_local_var_array_ir(
    output: &mut File,
    info: &mut GenerateIrInfo,
    dims: &[i32],
    result: &Vec<i32>,
    now_pos: i32,
    ptr: String,
) {
    if dims.is_empty() {
        //到达叶子
        if result[now_pos as usize] == 0 {
            writeln!(output, "  store 0, {}", ptr).unwrap();
        } else {
            writeln!(output, "  store %{}, {}", result[now_pos as usize], ptr).unwrap();
        }
    } else {
        //未到达叶子
        //计算增量，是dims[1..]的乘积
        let mut delta: i32 = 1;
        for dim in &dims[1..] {
            delta *= dim;
        }

        for i in 0..dims[0] {
            info.now_id += 1;
            writeln!(output, "  %{} = getelemptr {}, {}", info.now_id, ptr, i).unwrap();
            gen_local_var_array_ir(
                output,
                info,
                &dims[1..],
                result,
                now_pos + delta * i,
                "%".to_owned() + info.now_id.to_string().as_str(),
            );
        }
    }
}

///为局部常量数组生成代码
fn gen_local_const_array_ir(
    output: &mut File,
    info: &mut GenerateIrInfo,
    dims: &[i32],
    result: &Vec<i32>,
    now_pos: i32,
    ptr: String,
) {
    if dims.is_empty() {
        //到达叶子
        writeln!(output, "  store {}, {}", result[now_pos as usize], ptr).unwrap();
    } else {
        //未到达叶子
        //计算增量，是dims[1..]的乘积
        let mut delta: i32 = 1;
        for dim in &dims[1..] {
            delta *= dim;
        }

        for i in 0..dims[0] {
            info.now_id += 1;
            writeln!(output, "  %{} = getelemptr {}, {}", info.now_id, ptr, i).unwrap();
            gen_local_const_array_ir(
                output,
                info,
                &dims[1..],
                result,
                now_pos + delta * i,
                "%".to_owned() + info.now_id.to_string().as_str(),
            );
        }
    }
}
