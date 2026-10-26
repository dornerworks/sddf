#
# Copyright 2026, DornerWorks
#
# SPDX-License-Identifier: BSD-2-Clause
#
# Include this snippet in your project Makefile to build
# the ZynqMP Ethernet driver
#
# NOTES
#  Generates eth_driver.elf

ETHERNET_DRIVER_DIR := $(dir $(lastword $(MAKEFILE_LIST)))
CHECK_NETDRV_FLAGS_MD5:=.netdrv_cflags-$(shell echo -- ${CFLAGS} ${CFLAGS_network} | shasum | sed 's/ *-//')

${CHECK_NETDRV_FLAGS_MD5}:
	-rm -f .netdrv_cflags-*
	touch $@

microkit_sdk_config_dir := $(MICROKIT_SDK)/board/$(MICROKIT_BOARD)/$(MICROKIT_CONFIG)
sel4_include_dirs := $(microkit_sdk_config_dir)/include

eth_driver.elf: $(build_dir)/eth_driver.elf.intermediate

.INTERMDIATE: $(build_dir)/eth_driver.elf.intermediate
$(build_dir)/eth_driver.elf.intermediate:
	SEL4_INCLUDE_DIRS=$(abspath $(sel4_include_dirs)) \
	PROJECT_ROOT=${LIONSOS} \
		cargo build \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--manifest-path ${ETHERNET_DRIVER_DIR}/eth_driver/Cargo.toml

-include zymqmp/ethernet.d
