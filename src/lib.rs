//
//  SOS: the Stupid Operating System
//  by Hawk Weisman (hi@hawkweisman.me)
//
//  Copyright (c) 2015 Hawk Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! SOS: the Stupid Operating System
//!
//! SOS is a simple, tiny toy OS implemented in Rust.
//!
//! I'm writing this mostly for fun, to learn more about OS design and kernel //! hacking, so don't expect anything new or exciting out of this project.

#![crate_name = "sos_kernel"]
#![crate_type = "staticlib"]
#![feature( no_std
          , lang_items
          , asm )]
#![feature( const_fn
          , core_slice_ext
          , slice_patterns
          , iter_cmp
          )]
#![feature(collections)]

#![no_std]

extern crate collections;
extern crate rlibc;
extern crate spin;

extern crate sos_multiboot2 as multiboot;
extern crate sos_alloc as alloc;

#[macro_use] extern crate sos_vga as vga;
#[macro_use] extern crate bitflags;

pub mod arch;
#[macro_use] pub mod io;
pub mod util;
pub mod panic;
pub mod memory;

use arch::cpu;

/// Kernel main loop
pub fn kernel_main() {
    loop { }
}

/// Kernel initialization function called from ASM
///
/// The kernel main loop expects to be passed the address of a valid
/// Multiboot 2 info struct. It's the bootloader's responsibility to ensure
/// that this is passed in the correct register as expected by the calling
/// convention (`edi` on x86). If this isn't there, you can expect to have a
/// bad problem and not go to space today.
#[no_mangle]
pub extern fn kernel_start(multiboot_addr: usize) {
    io::term::CONSOLE.lock().clear();

    println!("Hello from the kernel!");

    // -- Unpack multiboot tag ------------------------------------------------
    let boot_info = unsafe { multiboot::Info::from(multiboot_addr) };

    let mmap_tag // Extract the memory map tag from the multiboot info
        = boot_info.mem_map()
                   .expect("Memory map tag required!");

    println!("Detected memory areas:");
    for a in mmap_tag.areas() {
        println!("     start: {:#08x}, end: {:#08x}"
                , a.base, a.length );
    }

    let elf_sections_tag // Extract ELF sections tag from the multiboot info
        = boot_info.elf64_sections()
                   .expect("ELF sections tag required!");

    println!("Detecting kernel ELF sections:");

    let kernel_begin    // Extract kernel ELF sections from  multiboot info
        = elf_sections_tag.sections()
            .map(|s| {
                println!("     address: {:#08x}, size: {:#08x}, flags: {:#08x}"
                        , s.address
                        , s.length
                        , s.flags );
                s.address })
            .min()
            .expect("Could not find kernel start section!\
                    \nSomething is deeply wrong.");

    let mut n_elf_sections = 0;
    let kernel_end
        = elf_sections_tag.sections()
            .map(|s| { n_elf_sections += 1;
                     s.address })
            .max()
            .expect("Could not find kernel end section!\
                    \nSomething is deeply wrong.");

    println!( "Detected {} kernel ELF sections.", n_elf_sections);
    println!( "Kernel begins at {:#x} and ends at {:#x}."
             , kernel_begin, kernel_end );

    let multiboot_end = multiboot_addr + boot_info.length as usize;

    println!( "Multiboot info begins at {:#x} and ends at {:#x}."
             , multiboot_addr, multiboot_end);

    // -- initialize interrupts ----------------------------------------------
    unsafe {
        println!("Intializing interrupts...");
        cpu::interrupts::initialize();
        println!("Intializing interrupts...    [DONE]" );
    };

    // -- initialize the heap ------------------------------------------------
    unsafe {
        print!("Intializing heap...    ");
        memory::init_heap();
        println!( "[DONE]\nHeap begins at {:#x} and ends at {:#x}."
                , memory::heap_base_addr(), memory::heap_top_addr() );
    };


    // let mut a_vec = collections::vec::Vec::<usize>::new();
    // println!( "TEST: Created a vector in kernel space! {:?}", a_vec);
    // a_vec.push(1);
    // println!( "TEST: pushed to vec: {:?}", a_vec);
    // a_vec.push(2);
    // println!( "TEST: pushed to vec: {:?}", a_vec);

    // -- call into kernel main loop ------------------------------------------
    // (currently, this does nothing)
    kernel_main()
}
