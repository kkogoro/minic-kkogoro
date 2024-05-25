# Learn Rust in a miniC compiler project

一边学习Rust一边实现的SysY to RISC-V编译器。

由于我是Rust初学者,代码中有很多不地道的写法,我还在修正中。

前端从SysY生成Koopa IR,后端从Koopa IR生成RISC-V。

前端部分暂时使用文本形式IR,或许暑假会改成内存形式。

## 目前已完成

- [x] 前端

- [x] 后端


## 其他问题

- [x] 还是可以构造出冲突的变量名，可以在ds_for_ir中单独实现一个查询函数名称的接口（通过符号表查询）。

- [ ] 函数名的查询接口已经实现了，但是main和库函数的名字不应该被混淆，修正方式是在全局符号表`tables[0]`中的函数名不做混淆，等出问题再改。

- [ ] 将gen_ir拆分成多个模块，等我学完rust模块管理再改。

- [ ] 把寄存器当作内存的cache

- [ ] 下面代码会出现IR缺换行的问题

```cpp

int main()
{
    int a[2][3];
    return 0;
}
```