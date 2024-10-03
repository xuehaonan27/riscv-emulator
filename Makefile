SIM = target/release/riscv-emulator
TARGET = test/build/$(T).bin
CARGO = cargo

all:
	@echo "-------Build Simulator-------"
	@cargo build --release
	@echo "-------Build Test-------"
	@$(MAKE) -C test T=$(T)
	@echo "-------Start Simulation-------"
	$(SIM) -i ./test/build/$(T).elf

clean:
	@$(MAKE) -C sim clean
	@$(MAKE) -C test clean

.PHONY: clean all
