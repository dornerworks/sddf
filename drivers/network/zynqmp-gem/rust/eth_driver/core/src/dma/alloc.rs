//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ptr::NonNull;

pub struct GemDmaPtrs {
    pub rx: DmaPtrs,
    pub tx: DmaPtrs,
    pub tx_dummy: DmaPtr,
}

pub struct DmaPtrs {
    pub desc: DmaPtr,
}

pub struct DmaPtr {
    pub vaddr: *mut (),
    pub paddr: *mut (),
}

pub struct DmaDef {
    pub vaddr: NonNull<()>,
    pub paddr: NonNull<()>,
    pub size: usize,
}
