#
# Copyright 2026, Skykraft
#
# SPDX-License-Identifier: BSD-2-Clause
#
# Include this snippet in your project Makefile to build
# the zynqmp-gem NIC driver
#
# There are two implementations of this driver, selected with NETWORK_DRIVER:
#    c:    (default) the C driver in c/.
#    rust: the Rust driver in rust/.
#    pancake: the Pancake driver in pancake/.
#
# NOTES
#  Generates eth_driver_znyqmp_gem.elf
#  Expects libsddf_util_debug.a to be in LIBS

NETWORK_DRIVER ?= c

ETHERNET_DRIVER_DIR := $(dir $(lastword $(MAKEFILE_LIST)))
CHECK_NETDRV_FLAGS_MD5:=.netdrv_cflags-$(shell echo -- ${CFLAGS} ${CFLAGS_network} | shasum | sed 's/ *-//')

${CHECK_NETDRV_FLAGS_MD5}:
	-rm -f .netdrv_cflags-*
	touch $@

ifeq ($(NETWORK_DRIVER),c)

eth_driver_znyqmp_gem.elf: network/zynqmp-gem/ethernet.o
	$(LD) $(LDFLAGS) $^ $(LIBS) -o $@

network/zynqmp-gem/ethernet.o: ${ETHERNET_DRIVER_DIR}/c/ethernet.c ${CHECK_NETDRV_FLAGS_MD5}
	mkdir -p network/zynqmp-gem
	${CC} -c ${CFLAGS} ${CFLAGS_network} -I ${ETHERNET_DRIVER_DIR}/c -o $@ $<

-include zynqmp-gem/ethernet.d

else ifeq ($(NETWORK_DRIVER),rust)

ETH_CARGO_TARGET_SPEC ?= $(SDDF)/support/targets/$(ARCH)-sel4-microkit-minimal.json
ETH_CARGO_TARGET_DIR := $(abspath cargo_target)
ETH_CARGO_ARTIFACT_DIR := $(ETH_CARGO_TARGET_DIR)/artifacts
ETH_CARGO_ELF := ${ETH_CARGO_ARTIFACT_DIR}/eth_driver_zynqmp.elf
ETH_CARGO_DOC ?= 0
ETH_CARGO_DOC_DIR := $(abspath ${ETHERNET_DRIVER_DIR}/rust/target)

ifeq ($(wildcard $(ETH_CARGO_TARGET_SPEC)),)
$(error No Rust target spec for ARCH=$(ARCH): $(ETH_CARGO_TARGET_SPEC))
endif

.PHONY: eth_driver_cargo
eth_driver_cargo:
	SEL4_INCLUDE_DIRS=$(abspath ${BOARD_DIR}/include) \
		cargo build --release \
			--target $(abspath $(ETH_CARGO_TARGET_SPEC)) \
			-Z json-target-spec \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			-Z unstable-options \
			--artifact-dir ${ETH_CARGO_ARTIFACT_DIR} \
			--target-dir ${ETH_CARGO_TARGET_DIR} \
			--manifest-path ${ETHERNET_DRIVER_DIR}/rust/eth_driver/Cargo.toml
ifeq ($(ETH_CARGO_DOC),1)
	SEL4_INCLUDE_DIRS=$(abspath ${BOARD_DIR}/include) \
		cargo doc \
			--config $(abspath ${ETH_CARGO_CONFIG}) \
			-Z json-target-spec \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--target-dir ${ETH_CARGO_DOC_DIR} \
			--manifest-path ${ETHERNET_DRIVER_DIR}/rust/eth_driver/Cargo.toml \
			--open
endif

eth_driver_znyqmp_gem.elf: eth_driver_cargo
	cmp -s ${ETH_CARGO_ELF} $@ || cp ${ETH_CARGO_ELF} $@

else ifeq ($(NETWORK_DRIVER),pancake)

ETH_DRIVER_PNK = \
	${SDDF}/util/util.pnk \
	${SDDF}/include/sddf/network/queue.pnk \
	${ETHERNET_DRIVER_DIR}/pancake/ethernet.pnk

eth_driver_znyqmp_gem.elf: network/zynqmp-gem/ethernet_pnk.o network/zynqmp-gem/ethernet.o \
		network/zynqmp-gem/ethernet_init_pnk.o util/pancake_ffi.o util/pancake_common.o
	$(LD) $(LDFLAGS) $^ $(LIBS) -o $@

network/zynqmp-gem/ethernet_pnk.o: network/zynqmp-gem/ethernet_pnk.s
	$(CC) -c $(CFLAGS) -o $@ $<

network/zynqmp-gem/ethernet_pnk.S: network/zynqmp-gem/ethernet_pnk.pnk
	$(PANCAKE_COMPILER) $(PANCAKE_FLAGS) < $< > $@

network/zynqmp-gem/ethernet_pnk.pnk: $(ETH_DRIVER_PNK)
	mkdir -p network/zynqmp-gem
	cat $^ | $(CPP) -P -CC -nostdinc > $@

network/zynqmp-gem/ethernet_pre.o: ${ETHERNET_DRIVER_DIR}/c/ethernet.c ${CHECK_NETDRV_FLAGS_MD5}
	mkdir -p network/zynqmp-gem
	${CC} -c ${CFLAGS} ${CFLAGS_network} -I ${ETHERNET_DRIVER_DIR}/c -o $@ $<

network/zynqmp-gem/ethernet.o: network/zynqmp-gem/ethernet_pre.o
	$(OBJCOPY) --redefine-sym init=c_init --redefine-sym notified=c_notified $< $@

network/zynqmp-gem/ethernet_init_pnk.o: ${ETHERNET_DRIVER_DIR}/pancake/ethernet_init_pnk.c ${CHECK_NETDRV_FLAGS_MD5}
	mkdir -p network/zynqmp-gem
	${CC} -c ${CFLAGS} ${CFLAGS_network} -I ${ETHERNET_DRIVER_DIR}/c -o $@ $<

clean::
	rm -f network/zynqmp-gem/ethernet_init_pnk.[do] network/zynqmp-gem/ethernet.o
	rm -f network/zynqmp-gem/ethernet_pre.[do] network/zynqmp-gem/ethernet_pnk.pnk
	rm -f network/zynqmp-gem/ethernet_pnk.[doS]

else
$(error NETWORK_DRIVER must be c, rust, or pancake, not '$(NETWORK_DRIVER)')
endif
