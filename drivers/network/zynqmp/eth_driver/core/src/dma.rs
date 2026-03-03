//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

mod alloc;
mod rx;
mod tx;

pub use alloc::{alloc_dma, DmaDef, DmaPtr, DmaPtrs, GemDmaPtrs};
pub use rx::RxRing;
pub use tx::{TxDummy, TxRing};

const NUM_BUFS: usize = 8;
pub const MTU: usize = 1600;
