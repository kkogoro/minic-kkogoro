pub mod ast;
pub mod calc_exp;
pub mod ds_for_ir;
mod gen_asm;
mod gen_ir;
pub mod symbol_table;

use gen_asm::GenerateAsm;
use gen_ir::GenerateIR;
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Result;
use std::io::Write;
// 引用 lalrpop 生成的解析器
// 因为我们刚刚创建了 sysy.lalrpop, 所以模块名是 sysy
lalrpop_mod!(sysy);

fn main() -> Result<()> {
    // 解析命令行参数
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();

    // 输出解析得到的 AST
    //let my_koppa_ir = format!("{}", ast);

    let mut output_file = File::create(output)?;

    match mode.as_str() {
        "-koopa" => {
            let mut info = ds_for_ir::GenerateIrInfo {
                now_id: 0,
                table: symbol_table::SymbolTable::new(),
            };
            ast.generate(&mut output_file, &mut info);
            //println!("{:#?}", ast);
            //writeln!(output_file, "{}", my_koppa_ir)?;
        }
        "-riscv" => {
            //let driver = koopa::front::Driver::from(my_koppa_ir);
            //let program = driver.generate_program().unwrap();
            //program.generate(&mut output_file);
        }
        _ => {
            panic!("Unknown mode: {}", mode);
        }
    }
    Ok(())
}
