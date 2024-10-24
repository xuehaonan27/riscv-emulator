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

ifeq ($(FT), enable)
	FTRACE = --ftrace
else
	FTRACE =
endif

ifeq ($(CPU), single)
	CPU_MODE = --cpu-mode single
else ifeq ($(CPU), multi)
	CPU_MODE = --cpu-mode multi
else ifeq ($(CPU), pipeline)
	CPU_MODE = --cpu-mode pipeline
else
	CPU_MODE = 
endif

ifeq ($(PRE_PIPELINE_INFO), enable)
	__PRE_PIPELINE_INFO = --pre-pipeline-info
else
	__PRE_PIPELINE_INFO = 
endif

ifeq ($(PIPELINE_INFO), enable)
	__PIPELINE_INFO = --pipeline-info
else
	__PIPELINE_INFO =
endif

ifeq ($(POST_PIPELINE_INFO), enable)
	__POST_PIPELINE_INFO = --post-pipeline-info
else
	__POST_PIPELINE_INFO =
endif

ifeq ($(CONTROL_HAZARD_INFO), enable)
	__CONTROL_HAZARD_INFO = --control-hazard-info
else
	__CONTROL_HAZARD_INFO =
endif

ifeq ($(DATA_HAZARD_INFO), enable)
	__DATA_HAZARD_INFO = --data-hazard-info
else
	__DATA_HAZARD_INFO =
endif

all:
	@echo "-------Build Simulator-------"
	@$(CARGO) build --release
	@echo "-------Build Test-------"
	@$(MAKE) -C test T=$(T)
	@echo "-------Start Simulation-------"
	$(SIM) $(DEBUG) $(ITRACE) $(MTRACE) $(FTRACE) \
	$(__PRE_PIPELINE_INFO) \
	$(__PIPELINE_INFO) \
	$(__POST_PIPELINE_INFO) \
	$(__CONTROL_HAZARD_INFO) \
	$(__DATA_HAZARD_INFO) \
	$(CPU_MODE) -i ./test/build/$(T).elf

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
	-@$(SIM) $(CPU_MODE) -i ./test/build/$@.elf


clean:
	@$(CARGO) clean
	@$(MAKE) -C test clean

.PHONY: clean all
