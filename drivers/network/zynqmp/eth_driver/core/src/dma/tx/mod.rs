//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::{DmaPtr, DmaPtrs, NUM_BUFS};
use core::ops::{Deref, DerefMut};

mod descriptor;
use descriptor::Descriptor;

pub struct TxDummy {
    desc: *mut Descriptor,
}

impl TxDummy {
    pub fn new(dma_ptr: &DmaPtr) -> Self {
        let mut tx_dummy = Self {
            desc: dma_ptr.vaddr.cast(),
        };
        tx_dummy.setup();
        tx_dummy
    }

    fn setup(&mut self) {
        self.clear_status();
        self.mark_sw_owned();
        self.mark_last();
    }
}

impl Deref for TxDummy {
    type Target = Descriptor;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.desc }
    }
}

impl DerefMut for TxDummy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.desc }
    }
}

pub struct TxRing {
    head: usize,
    tail: usize,
    entries: *mut [Descriptor; NUM_BUFS],
}

impl TxRing {
    pub fn new(dma_ptrs: &DmaPtrs) -> Self {
        let entries = dma_ptrs.desc.vaddr.cast();
        let mut ring = Self {
            head: 0,
            tail: 0,
            entries,
        };
        ring.setup();
        ring
    }

    fn entries(&self) -> *mut [Descriptor; NUM_BUFS] {
        self.entries
    }

    pub fn get_tail(&self) -> usize {
        self.tail
    }

    fn setup(&mut self) {
        for (_i, entry) in self.iter_mut().enumerate() {
            entry.mark_sw_owned();
        }
        self.last_mut().unwrap().mark_last();
    }

    pub fn entry_available(&self) -> bool {
        let entries_len = self.len();
        let index = self.tail % entries_len;
        self.get(index).unwrap().is_available()
    }

    pub fn set_desc(&mut self, buffer_paddr: usize, len: usize) {
        let entries_len = self.len();
        let index = self.tail % entries_len;
        let desc = self.get_mut(index).unwrap();
        desc.clear_status();
        desc.set_len(len);
        desc.set_addr(buffer_paddr);
        // Assume only single buffer sized frames
        desc.mark_frame_end();
        desc.mark_gem_owned();

        self.tail +=1;
    }

    pub fn get_buffer(&mut self) -> usize {
        let entries_len = self.len();
        let index = self.head % entries_len;
        let entry = self.get_mut(index).unwrap();
        entry.mark_sw_owned();
        let addr = entry.addr() as usize;
        self.head += 1;
        addr
    }

    pub fn is_empty(&self) -> bool {
        let len = self.tail - self.head;
        len == 0
    }

    pub fn is_full(&self) -> bool {
        let len = self.tail - self.head;
        len == self.len()
    }
}

impl Deref for TxRing {
    type Target = [Descriptor; NUM_BUFS];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.entries() }
    }
}

impl DerefMut for TxRing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.entries() }
    }
}
