#![#![no_std]
#![#![feature(allocator_api, global_asm)]]

// Kernel module imported. For the official macro documentation refers to https://rust-for-linux.github.io/docs/kernel/
//  TODO: check if some module can be removed 
use kernel::prelude::*;
use kernel::{
    device::RawDevice,
    file::{File, Operations},
    io_buffer::{IoBufferReader, IoBufferWriter},
};

module! {
    type: DriverVerifier,
    name: "driver_verifier",
    author: "Giorgio Saldana",
    description: "A kernel module to verify driver functionality",
    license: "GPL",
}


