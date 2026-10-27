# 实验一：多道程序与分时多任务

## 实现概述

为每个任务维护系统调用计数，在分发系统调用时先计数；实现 sys_trace 的字节读取、写入及按编号查询调用次数，包含本次 trace 调用。

## 问答

1. 使用 RustSBI 0.4.1 和 QEMU 8.2.2 运行第二章三个 bad 测例：非法地址访问触发 PageFault；特权指令和 S 态寄存器访问分别触发 IllegalInstruction，内核终止对应应用并继续运行。
2. `__restore` 入口的 sp 指向内核栈中的 TrapContext；既用于首次进入应用，也用于处理 trap 后恢复应用。sstatus 决定 sret 返回的特权级及中断状态，sepc 指向用户态恢复地址，sscratch 暂存用户 sp。x2 是 sp，最后单独交换恢复；x4 是 tp，框架未使用它。交换后 sp 是用户栈指针，sscratch 是内核栈顶。`sret` 根据 sstatus.SPP 返回 U 态。进入 `__alltraps` 时交换后 sp 是内核栈顶、sscratch 是用户 sp。U 态的 ecall 或其他异常、时钟中断触发硬件进入 S 态并跳转到 stvec。

## 验证

使用 QEMU 8.2.2 运行官方检查器，测试通过 7/7，报告文件检查通过。
