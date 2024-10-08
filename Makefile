SIM = target/release/riscv-emulator
CARGO = cargo

ifeq ($(M), debug)
    DEBUG = --debug
else
    DEBUG =
endif

ifeq ($(IT), enable)
	ITRACE = --itrace
else
	ITRACE =
endif

ifeq ($(MT), enable)
	MTRACE = --mtrace
else
	MTRACE =
endif

all:
	@echo "-------Build Simulator-------"
	@$(CARGO) build --release
	@echo "-------Build Test-------"
	@$(MAKE) -C test T=$(T)
	@echo "-------Start Simulation-------"
	$(SIM) $(DEBUG) $(ITRACE) $(MTRACE) -i ./test/build/$(T).elf

RUST_SRC := src
RUST_SRC_FILES := $(wildcard $(RUST_SRC)/*.rs)
SRC_DIR := test/src
SRC_FILES := $(wildcard $(SRC_DIR)/*.c)
FILE_NAMES := $(notdir $(basename $(SRC_FILES)))

sim: $(RUST_SRC_FILES)
	@$(CARGO) build --release

run: sim $(FILE_NAMES)

$(FILE_NAMES):
	@echo "-------Running $@-------"
	@echo "-------Build Test $@-------"
	@$(MAKE) -C test T=$@
	@echo "-------Start Simulation-------"
	-@$(SIM) $(ITRACE) -i ./test/build/$@.elf


clean:
	@$(CARGO) clean
	@$(MAKE) -C test clean

.PHONY: clean all
