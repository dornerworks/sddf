/*
 * Copyright 2026, DornerWorks
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

#include <stdint.h>
#include <microkit.h>
#include <sddf/resources/device.h>
#include <sddf/network/queue.h>
#include <sddf/network/config.h>
#include <sddf/util/pancake_common.h>
#include <sddf/util/printf.h>
#include "ethernet.h"

/* renamed from the init() and notified() function of the C driver */
extern void c_init(void);

extern device_resources_t device_resources;
extern net_driver_config_t config;
extern net_queue_handle_t rx_queue;
extern net_queue_handle_t tx_queue;
extern hw_ring_t rx;
extern hw_ring_t tx;
extern volatile zynqmp_gem_regs_t *eth;

void init(void)
{
    c_init();

    init_pancake_mem();

    uintptr_t *pnk_mem = (uintptr_t *)cml_heap;

    pnk_mem[0] = (uintptr_t)eth;
    pnk_mem[1] = device_resources.irqs[0].id;
    pnk_mem[2] = config.virt_rx.id;
    pnk_mem[3] = config.virt_tx.id;
    pnk_mem[4] = (uintptr_t)&rx_queue;
    pnk_mem[5] = (uintptr_t)&tx_queue;
    pnk_mem[6] = (uintptr_t)&rx;
    pnk_mem[7] = (uintptr_t)&tx;

    cml_main();
}

void ffieth_rx_error(unsigned char *c, long clen, unsigned char *a, long alen)
{
    sddf_dprintf("ETH|ERROR: Receive error, status: %u\n", (uint32_t)clen);
}

void ffieth_tx_error(unsigned char *c, long clen, unsigned char *a, long alen)
{
    sddf_dprintf("ETH|ERROR: Transmit error, status: %u\n", (uint32_t)clen);
}

void ffieth_unexpected_notification(unsigned char *c, long clen, unsigned char *a, long alen)
{
    sddf_dprintf("ETH|LOG: Unexpected notification on channel: %u\n", (microkit_channel)clen);
}
