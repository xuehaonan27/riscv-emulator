# RISC-V Simulator
## Introduction
This is RISC-V simulator written in Rust Programming Language.

## Steps to run tests (For Lab2-2)
0. Get Rust toolchain and make sure you could compile Rust codes with `cargo`.
1. Clone the repository: `git clone https://github.com/xuehaonan27/riscv-simulator`.
2. Enter the directory: `cd riscv-simulator`.
3. Checkout to submit branch. `git checkout lab2-2`.
4. Run tests
```shell
# Run tests on single cycle CPU.
make run CPU=single

# Run tests on multi-stage CPU.
make run CPU=multi

# Run tests on 5-stages pipeline CPU.
# Stall when data hazard encountered.
# Static branch prediction: always not taken.
make run CPU=pipeline DATA_HAZARD_POLICY=naiveStall  CONTROL_POLICY=alwaysNotTaken

# Run tests on 5-stages pipeline CPU.
# Stall when data hazard encountered.
# Dynamic branch prediction: 1-bit predictor.
make run CPU=pipeline DATA_HAZARD_POLICY=naiveStall CONTROL_POLICY=dynamicPredict PREDICT_POLICY=oneBit 

# Run tests on 5-stages pipeline CPU.
# Stall when data hazard encountered.
# Dynamic branch prediction: 2-bit predictor.
make run CPU=pipeline DATA_HAZARD_POLICY=naiveStall CONTROL_POLICY=dynamicPredict PREDICT_POLICY=twoBits 

# Run tests on 5-stages pipeline CPU.
# Data forward when data hazard encountered.
# Static branch prediction: always not taken.
make run CPU=pipeline DATA_HAZARD_POLICY=dataForward  CONTROL_POLICY=alwaysNotTaken

# Run tests on 5-stages pipeline CPU.
# Data forward when data hazard encountered.
# Dynamic branch prediction: 1-bit predictor.
make run CPU=pipeline DATA_HAZARD_POLICY=dataForward CONTROL_POLICY=dynamicPredict PREDICT_POLICY=oneBit 

# Run tests on 5-stages pipeline CPU.
# Data forward when data hazard encountered.
# Dynamic branch prediction: 2-bit predictor.
make run CPU=pipeline DATA_HAZARD_POLICY=dataForward CONTROL_POLICY=dynamicPredict PREDICT_POLICY=twoBits
```
5. Run a single test.
```shell
make T=add \
    CPU=pipeline \
    PRE_PIPELINE_INFO=enable \
    PIPELINE_INFO=enable \
    POST_PIPELINE_INFO=enable \
    CONTROL_HAZARD_INFO=enable \
    DATA_HAZARD_INFO=enable \
    DATA_HAZARD_POLICY=dataForward \
    CONTROL_POLICY=dynamicPredict \
    PREDICT_POLICY=twoBits
```
+ T: test name.
  + Available: `ackermann`, `add`, `div`, `dummy`, `if-else`, `load-store`, `matrix-mul`, `quicksort`, `shift`, `test`, `unalign`.
+ CPU: CPU types.
  + Available: `single`, `multi`, `pipeline`.
  + If `pipeline` CPU is used, **YOU MUST** specify **DATA_HAZARD_POLICY** and **CONTROL_POLICY**.
+ DATA_HAZARD_POLICY: policy for data hazard.
  + Available: `naiveStall`, `dataForward`.
+ CONTROL_POLICY: policy for control hazard.
  + Available: `alwaysNotTaken`, `dynamicPredict`.
  + If `dynamicPredict` is used, **YOU MUST** specify **PREDICT_POLICY**.
+ PREDICT_POLICY: policy for branch prediction.
  + Available: `oneBit`, `twoBits`. (one-bit predictor / two-bits predictor).
+ PRE_PIPELINE_INFO: pipeline registers information before this cycle's execution. Assign `enable` to enable.
+ POST_PIPELINE_INFO: pipeline registers information after this cycle's execution. Assign `enable` to enable.
+ CONTROL_HAZARD_INFO: control hazard information. Assign `enable` to enable.
+ DATA_HAZARD_INFO: data hazard information. Assign `enable` to enable.

## Steps to run tests (For Lab2-1)
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
