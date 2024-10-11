# RISC-V Simulator
## Introduction
This is RISC-V simulator written in Rust Programming Language.

## Steps to run tests
0. Get Rust toolchain and make sure you could compile Rust codes with `cargo`.
1. Clone the repository: `git clone https://github.com/xuehaonan27/riscv-simulator`.
2. Enter the directory: `cd riscv-simulator`.
3. Checkout to submit branch. `git checkout lab2-1`.
4. Run `make run` to run all tests.
5. Run `make T=<test_name>` to run a single test.
6. Run `make M=debug T=<test_name>` to run a single test with debugger.
7. Note: add `IT=enable` to make command to enable itrace.
8. Note: add `MT=enable` to make command to enable mtrace.
9. Note: add `FT=enable` to make command to enable ftrace.
