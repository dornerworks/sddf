//
// Copyright 2024, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::NUM_BUFS;
use core::ops::{Deref, DerefMut};

mod descriptor;
use descriptor::Descriptor;

use super::DmaPtrs;

pub struct RxRing {
    head: usize,
    tail: usize,
    entries: *mut [Descriptor; NUM_BUFS],
}

impl RxRing {
    pub fn new(dma_ptrs: &DmaPtrs) -> Self {
        let entries = dma_ptrs.desc.vaddr.cast();
        let ring = Self {
            head: 0,
            tail: 0,
            entries,
        };
        ring
    }

    fn entries(&self) -> *mut [Descriptor; NUM_BUFS] {
        self.entries
    }

    pub fn next_entry_available(&self) -> bool {
        let entries_len = self.len();
        let index = self.head % entries_len;
        self.get(index).unwrap().is_available()
    }

    pub fn recv_next(&mut self) -> (usize, u32) {
        let entries_len = self.len();
        let index = self.head % entries_len;
        let entry = self.get(index).unwrap();
        let addr = entry.addr();
        let len = entry.len();
        self.head += 1;
        (addr, len)
    }

    pub fn mark_done(&mut self, buffer_paddr: usize) {
        let entries_len = self.len();
        let index = self.tail % entries_len;
        let entry = self.get_mut(index).unwrap();
        entry.set_addr(buffer_paddr);
        entry.mark_done();
        if index == (NUM_BUFS - 1) {
            entry.mark_last();
        }
        self.tail += 1;
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

impl Deref for RxRing {
    type Target = [Descriptor; NUM_BUFS];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.entries() }
    }
}

impl DerefMut for RxRing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.entries() }
    }
}
