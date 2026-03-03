//
// Copyright 2026, DornerWorks
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub mod channels {
    use sel4_microkit::Channel;

    pub const DEVICE: Channel = Channel::new(0);
    pub const CLIENT: Channel = Channel::new(1);
}

pub mod log {
    use sel4_logging::{LevelFilter, Logger, LoggerBuilder};
    use sel4_microkit::debug_print;

    const LOG_LEVEL: LevelFilter = {
        // LevelFilter::Trace
        // LevelFilter::Debug
        LevelFilter::Info
        // LevelFilter::Warn
    };

    pub static LOGGER: Logger = LoggerBuilder::const_default()
        .level_filter(LOG_LEVEL)
        .write(|s| debug_print!("{}", s))
        .build();
}

const SDDF_NET_MAGIC_LEN: usize = 5;

#[repr(C)]
#[derive(Clone,Copy)]
pub struct region_resource_t {
    pub vaddr : *mut (),
    pub size : u64
}

#[repr(C)]
pub struct net_connection_resource_t {
    pub free_queue: region_resource_t,
    pub active_queue : region_resource_t,
    pub num_buffers : u16,
    pub id : u8
}

#[repr(C)]
pub struct net_driver_config_t {
    pub magic : [core::ffi::c_char; SDDF_NET_MAGIC_LEN],
    pub virt_rx : net_connection_resource_t,
    pub virt_tx : net_connection_resource_t,
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".net_driver_config")]
pub static mut NET_CONFIG: net_driver_config_t = net_driver_config_t {
    magic: [b'\0'; 5],
    virt_rx: net_connection_resource_t {
        free_queue: region_resource_t {
            vaddr: core::ptr::null_mut(),
            size: 0
        },
        active_queue: region_resource_t {
            vaddr: core::ptr::null_mut(),
            size: 0
        },
        num_buffers: 0,
        id: 0
    },
    virt_tx: net_connection_resource_t {
        free_queue: region_resource_t {
            vaddr: core::ptr::null_mut(),
            size: 0
        },
        active_queue: region_resource_t {
            vaddr: core::ptr::null_mut(),
            size: 0
        },
        num_buffers: 0,
        id: 0
    }
};

const DEVICE_MAX_REGIONS: usize = 64;
const DEVICE_MAX_IRQS: usize = 64;

#[repr(C)]
#[derive(Clone,Copy)]
pub struct device_region_resource_t {
    pub region : region_resource_t,
    pub io_addr : usize
}

#[repr(C)]
#[derive(Clone,Copy)]
pub struct device_irq_resource_t {
    pub id : u8
}

#[repr(C)]
pub struct device_resources_t {
    pub magic : [core::ffi::c_char; SDDF_NET_MAGIC_LEN],
    pub num_regions : u8,
    pub num_irqs : u8,
    pub regions : [device_region_resource_t; DEVICE_MAX_REGIONS],
    pub irqs : [device_irq_resource_t; DEVICE_MAX_IRQS]
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".device_resources")]
pub static mut DEVICE_RESOURCES: device_resources_t = device_resources_t {
    magic: [b'\0'; 5],
    num_regions: 0,
    num_irqs: 0,
    regions: [device_region_resource_t {
        region: region_resource_t {
            vaddr: core::ptr::null_mut(),
            size: 0
        },
        io_addr: 0
    }; DEVICE_MAX_REGIONS],
    irqs: [device_irq_resource_t {
        id: 0
    }; DEVICE_MAX_IRQS]
};

