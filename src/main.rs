#[macro_use]
//使用用于debug的宏
mod debug_macros;

pub mod ast;
pub mod calc_exp;
pub mod ds_for_ir;
pub mod symbol_table;

#[cfg(feature = "generate-asm")]
mod gen_asm;
#[cfg(feature = "generate-asm")]
use gen_asm::GenerateAsm;

#[cfg(feature = "generate-ir")]
mod gen_ir;
#[cfg(feature = "generate-ir")]
use gen_ir::GenerateIR;

use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Result;

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
            let mut info = ds_for_ir::GenerateIrInfo::new();

            symbol_table_debug!(
                "程序开始,符号表和block表分别为{:#?}\n{:#?}",
                info.tables,
                info.block_id
            );

            #[cfg(feature = "print-AST")]
            println!("{:#?}", ast);

            #[cfg(feature = "generate-ir")]
            {
                ast.generate(&mut output_file, &mut info);
            }
        }
        "-riscv" => {
            #[cfg(feature = "generate-asm")]
            {
                let driver = koopa::front::Driver::from(my_koppa_ir);
                let program = driver.generate_program().unwrap();
                program.generate(&mut output_file);
            }
        }
        _ => {
            panic!("Unknown mode: {}", mode);
        }
    }
    Ok(())
}
