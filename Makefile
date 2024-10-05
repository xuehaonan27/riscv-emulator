SIM = target/release/riscv-emulator
TARGET = test/build/$(T).bin
CARGO = cargo

ifeq ($(M), debug)
    DEBUG = --debug
else
    DEBUG =
endif

all:
	@echo "-------Build Simulator-------"
	@cargo build --release
	@echo "-------Build Test-------"
	@$(MAKE) -C test T=$(T)
	@echo "-------Start Simulation-------"
	$(SIM) $(DEBUG) -i ./test/build/$(T).elf

clean:
	@$(MAKE) -C sim clean
	@$(MAKE) -C test clean

.PHONY: clean all
