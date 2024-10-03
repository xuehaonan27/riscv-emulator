## Benches
### Dhrystone
Compilation flags: -Wall -march=rv64ifd -std=c90

### whetstone
Compilation: `riscv64-unknown-elf-gcc -march=rv64ifd -Wall whetstone.c -lm -o whetstone`

### CNN inference
Compilation: `riscv64-unknown-elf-g++ -march=rv64ifd -Wall lab1_cnn_inference.cpp -o lab1_cnn_inference`