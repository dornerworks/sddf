/*
 * Copyright 2026, DornerWorks
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */

#include <stdint.h>
#include <microkit.h>
#include <sddf/util/pancake_common.h>

/* renamed from the init() and notified() function of the C driver */
extern void c_init(void);
extern void c_notified(microkit_channel ch);

void init(void)
{
    c_init();

    init_pancake_mem();

    cml_main();
}

void ffic_notified(unsigned char *c, long clen, unsigned char *a, long alen)
{
    c_notified(clen);
}
