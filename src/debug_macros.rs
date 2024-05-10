//!定义了调试信息宏

///符号表的调试宏
#[cfg(feature = "symbol-table-debug")]
macro_rules! symbol_table_debug {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}
#[cfg(not(feature = "symbol-table-debug"))]
macro_rules! symbol_table_debug {
    ($($arg:tt)*) => {};
}
