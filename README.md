# RISC-V Simulator
## Introduction
This is RISC-V simulator written in Rust Programming Language.

## Steps to run tests
0. Get Rust toolchain and make sure you could compile Rust codes with `cargo`.
1. Clone the repository: `git clone https://github.com/xuehaonan27/riscv-simulator`.
2. Enter the directory: `cd riscv-simulator`.
3. Run `make run` to run all tests.
4. Run `make T=<test_name>` to run a single test.
5. Run `make M=debug T=<test_name>` to run a single test with debugger.

## Benches
### Dhrystone
Compilation flags: -Wall -march=rv64ifd -std=c90

### whetstone
Compilation: `riscv64-unknown-elf-gcc -march=rv64ifd -Wall whetstone.c -lm -o whetstone`

### CNN inference
Compilation: `riscv64-unknown-elf-g++ -march=rv64ifd -Wall lab1_cnn_inference.cpp -o lab1_cnn_inference`
