//
// Copyright 2024, DornerWorks
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]
#![no_main]

use core::ptr::NonNull;

use eth_driver_core::Driver;
use log::{info, debug};
use sel4_driver_interfaces::HandleInterrupt;
use sel4_shared_memory::SharedMemoryRef;
use sel4_microkit::{protection_domain, Channel, ChannelSet, Handler, Infallible, MessageInfo};
use sel4_sddf_netqueue::{NetQueueHandle, RawNetQueue, NetBuffDesc};

use crate::config::NET_CONFIG;
use crate::config::DEVICE_RESOURCES;

mod config;

#[protection_domain]
fn init<'a>() -> HandlerImpl {
    config::log::LOGGER.set().unwrap();
    info!("Running Ethernet Driver");
    let rx_notify: fn() = || config::channels::DEVICE.notify();
    let tx_notify: fn() = || config::channels::CLIENT.notify();

    debug!("RX Free Vaddr is   {:#?}", unsafe {NET_CONFIG.virt_rx.free_queue.vaddr  });
    debug!("RX Free size  is   {:x }", unsafe {NET_CONFIG.virt_rx.free_queue.size   });
    debug!("RX Free Vaddr is   {:#?}", unsafe {NET_CONFIG.virt_rx.free_queue.vaddr  });
    debug!("RX Free size  is   {:x }", unsafe {NET_CONFIG.virt_rx.free_queue.size   });
    debug!("RX Active Vaddr is {:#?}", unsafe {NET_CONFIG.virt_rx.active_queue.vaddr});
    debug!("RX Active size is  {:x }", unsafe {NET_CONFIG.virt_rx.active_queue.size });
    debug!("RX Num Buffers is  {:x }", unsafe {NET_CONFIG.virt_rx.num_buffers });
    debug!("RX ID is           {:x }", unsafe {NET_CONFIG.virt_rx.id });

    debug!("TX Free Vaddr is   {:#?}", unsafe {NET_CONFIG.virt_tx.free_queue.vaddr  });
    debug!("TX Free size  is   {:x }", unsafe {NET_CONFIG.virt_tx.free_queue.size   });
    debug!("TX Active Vaddr is {:#?}", unsafe {NET_CONFIG.virt_tx.active_queue.vaddr});
    debug!("TX Active size is  {:x }", unsafe {NET_CONFIG.virt_tx.active_queue.size });
    debug!("TX Num Buffers is  {:x }", unsafe {NET_CONFIG.virt_tx.num_buffers });
    debug!("TX ID is           {:x }", unsafe {NET_CONFIG.virt_tx.id });

    debug!("rx_notify is {:#?}", rx_notify);
    debug!("DEVICE is {:#?}", config::channels::DEVICE);
    debug!("tx_notify is {:#?}", tx_notify);
    debug!("CLIENT is {:#?}", config::channels::CLIENT);

    debug!("DEVICE_RESOURCES {} vaddr {:#?}", 0, unsafe { DEVICE_RESOURCES.regions[0].region.vaddr });
    debug!("DEVICE_RESOURCES {} vaddr {:#?}", 1, unsafe { DEVICE_RESOURCES.regions[1].region.vaddr });
    debug!("DEVICE_RESOURCES {} vaddr {:#?}", 2, unsafe { DEVICE_RESOURCES.regions[2].region.vaddr });

    let rx_free_ptr = NonNull::new(unsafe {NET_CONFIG.virt_rx.free_queue.vaddr} as *mut RawNetQueue);
    let rx_active_ptr = NonNull::new(unsafe {NET_CONFIG.virt_rx.active_queue.vaddr} as *mut RawNetQueue);
    let rx_free = unsafe {
        SharedMemoryRef::new(
            rx_free_ptr.unwrap_or_else(|| { panic!("!{} is null", stringify!(rx_free_ptr)) })
        )
    };
    let rx_active = unsafe {
        SharedMemoryRef::new(
            rx_active_ptr.unwrap_or_else(|| { panic!("!{} is null", stringify!(rx_active_ptr)) })
        )
    };
    let rx_num_buffers = unsafe { NET_CONFIG.virt_rx.num_buffers };
    let rx_handle = NetQueueHandle::from_ptrs(rx_free, rx_active, rx_num_buffers.into(), false);

    let tx_free_ptr = NonNull::new(unsafe {NET_CONFIG.virt_tx.free_queue.vaddr} as *mut RawNetQueue);
    let tx_active_ptr = NonNull::new(unsafe {NET_CONFIG.virt_tx.active_queue.vaddr} as *mut RawNetQueue);
    let tx_free = unsafe {
        SharedMemoryRef::new(
            tx_free_ptr.unwrap_or_else(|| { panic!("!{} is null", stringify!(tx_free_ptr)) })
        )
    };
    let tx_active = unsafe {
        SharedMemoryRef::new(
            tx_active_ptr.unwrap_or_else(|| { panic!("!{} is null", stringify!(tx_active_ptr)) })
        )
    };
    let tx_num_buffers = unsafe { NET_CONFIG.virt_tx.num_buffers };
    let tx_handle = NetQueueHandle::from_ptrs(tx_free, tx_active, tx_num_buffers.into(), false);

    let mut dev = {
        Driver::new(
            unsafe { DEVICE_RESOURCES.regions[0].region.vaddr },
            unsafe { DEVICE_RESOURCES.regions[1].region.vaddr },
            unsafe { DEVICE_RESOURCES.regions[2].region.vaddr },
            unsafe { DEVICE_RESOURCES.regions[3].region.vaddr },
            unsafe { DEVICE_RESOURCES.regions[1].io_addr as *mut () },
            unsafe { DEVICE_RESOURCES.regions[2].io_addr as *mut () },
            unsafe { DEVICE_RESOURCES.regions[3].io_addr as *mut () },
        )
    };

    info!("Finished Initializing Driver");
    dev.handle_interrupt();
    info!("Acked driver IRQ");
    config::channels::DEVICE.irq_ack().unwrap();
    info!("Acked physical IRQ");

    HandlerImpl {
        drv: dev,
        rx: rx_handle,
        tx: tx_handle,
        client_channel: config::channels::CLIENT,
        device_channel: config::channels::DEVICE,
    }
}

struct HandlerImpl {
    drv: Driver,
    rx: NetQueueHandle<'static>,
    tx: NetQueueHandle<'static>,
    client_channel: sel4_microkit::Channel,
    device_channel: sel4_microkit::Channel,
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        if channels.contains(self.client_channel) || channels.contains(self.device_channel) {
            let mut notify_client = false;
            loop {
                if !self.drv.rx_is_full() && !self.rx.free.is_empty() {
                    let mut buffer: NetBuffDesc = NetBuffDesc::new(0, 0);
                    self.rx.free.dequeue(&mut buffer);
                    self.drv.rx_mark_done(buffer.io_or_offset as usize);
                } else {
                    break;
                }
            }

            if !self.drv.rx_is_full() {
                self.rx.free.request_signal();
            } else {
                self.rx.free.cancel_signal();
            }

            // TODO: Split up device handling vs client handling?
            for _ in 0..self.rx.capacity {
                match self.drv.receive() {
                    Some(packet) => {
                        let buffer = NetBuffDesc::new(packet.0 as u64, packet.1 as u16);
                        let _ = self.rx.active.enqueue(buffer);
                    }
                    None => break,
                }

                if self.rx.active.require_signal() {
                    self.rx.active.cancel_signal();
                    notify_client = true;
                }
            }

            loop {
                if !self.drv.tx_is_empty() {
                    let mut buffer: NetBuffDesc = NetBuffDesc::new(0, 0);
                    buffer.set_io_or_offset(self.drv.tx_get_buffer() as u64);
                    buffer.set_len(0);
                    let _ = self.tx.free.enqueue(buffer);
                } else {
                    break;
                }

                if self.rx.active.require_signal() {
                    self.rx.active.cancel_signal();
                    notify_client = true;
                }
            }

            loop {
                if !self.drv.tx_is_full() && !self.tx.active.is_empty() {
                    let mut buffer: NetBuffDesc = NetBuffDesc::new(0, 0);
                    self.tx.active.dequeue(&mut buffer);
                    self.drv.transmit(buffer.io_or_offset as usize, buffer.len.into());
                } else {
                    break;
                }
            }

            self.tx.active.request_signal();

            if notify_client {
                self.client_channel.notify();
            }

            self.drv.handle_interrupt();
            self.device_channel.irq_ack().unwrap();
        }
        Ok(())
    }

    fn protected(
        &mut self,
        _channel: Channel,
        _msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        debug!("Shouldn't be in protected");
        unreachable!()
    }
}
