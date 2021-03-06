//
//  SOS: the Stupid Operating System
//  by Eliza Weisman (eliza@elizas.website)
//
//  Copyright (c) 2015-2017 Eliza Weisman
//  Released under the terms of the MIT license. See `LICENSE` in the root
//  directory of this repository for more information.
//
//! 64-bit IDT gate implementation
use ::segment;
use super::{GateFlags};
use super::super::{InterruptHandler, ErrorCodeHandler};

use core::{convert, mem, ops};
use core::marker::PhantomData;

impl GateFlags {

    /// Returns a new trap gate
    pub const fn new_trap() -> Self {
        GateFlags { bits: super::TRAP_GATE_32.bits | super::PRESENT.bits }
    }

    /// Returns a new call gate
    pub const fn new_task() -> Self {
        GateFlags { bits: super::TASK_GATE_32.bits | super::PRESENT.bits }
    }

    /// Returns a new interrupt gate
    pub const fn new_interrupt() -> Self {
        GateFlags { bits: super::INT_GATE_32.bits | super::PRESENT.bits }
    }

}


/// An IDT entry is called a gate.
///
/// Based on [code](http://wiki.osdev.org/Interrupt_Descriptor_Table#Structure)
/// from the OS Dev Wiki.
///
/// Refer also to "6.14.1 64-Bit Mode IDT"  and "Table 3-2. System-Segment and
/// Gate-Descriptor Types" in the _Intel® 64 and IA-32 Architectures
/// Software Developer’s Manual_
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct Gate<H = InterruptHandler>
    { /// bits 0 - 15 of the offset
      pub offset_lower: u16
    , /// code segment selector (GDT or LDT)
      pub selector: segment::Selector
    , /// always zero
      _zero: u8
    , /// indicates the gate's type and attributes.
      /// the second half indicates the type:
      ///   + `0b1100`: Call gate
      ///   + `0b1110`: Interrupt gate
      ///   + `0b1111`: Trap Gate
      pub flags: GateFlags
    , /// bits 16 - 31 of the offset
      pub offset_mid: u16
    , /// bits 32 - 63 of the offset
      pub offset_upper: u32
    , /// always zero (according to the spec, this is "reserved")
      _reserved: u32
    , _handler_type: PhantomData<H>
    }

impl<H> Gate<H> {

    /// Creates a new IDT gate marked as `absent`.
    ///
    /// This is basically just for filling the new IDT table
    /// with valid (but useless) gates upon init.
    ///
    /// Actually triggering an absent interrupt will send a General Protection
    /// fault (13).
    pub const fn absent() -> Self {
       Gate { offset_lower: 0
            , selector: segment::Selector::from_raw(0)
            , _zero: 0
            , flags: GateFlags { bits:  0b1000_1110 }
            , offset_mid: 0
            , offset_upper: 0
            , _reserved: 0
            , _handler_type: PhantomData
            }
    }

    /// Set the handler function corresponding to this gate.
    #[inline]
    pub fn set_handler<F>(&mut self, handler: F) -> &mut Self
    where Self: convert::From<F> {
        *self = Self::from(handler);
        self
    }

    /// Sets the TRAP GATE flag to true
    #[inline]
    pub fn set_trap(&mut self) -> &mut Self {
        self.flags.insert(super::TRAP_GATE_32);
        self
    }

}


impl<H> Default for Gate<H> {
    fn default() -> Self {
        Gate { offset_lower: 0
             , selector: segment::Selector::from_raw(0)
             , _zero: 0
             , flags: GateFlags { bits: 0b1000_1110 }
             , offset_mid: 0
             , offset_upper: 0
             , _reserved: 0
             , _handler_type: PhantomData
             }
    }
}

impl<H> ops::Deref for Gate<H> {
    type Target = GateFlags;

    #[inline] fn deref(&self) -> &Self::Target { &self.flags }
}

impl<H> ops::DerefMut for Gate<H> {
    #[inline] fn deref_mut(&mut self) -> &mut Self::Target { &mut self.flags }
}

impl convert::From<InterruptHandler> for Gate<InterruptHandler> {

    /// Creates a new IDT gate pointing at the given handler function.
    ///
    /// The `handler` function must have been created with valid interrupt
    /// calling conventions.
    fn from(handler: InterruptHandler) -> Self {
        unsafe { // trust me on this, `mem::transmute()` is glorious black magic
                let (low, mid, high): (u16, u16, u32) = mem::transmute(handler);

            Gate { offset_lower: low
                 , offset_mid: mid
                 , offset_upper: high
                 , selector: segment::Selector::from_cs()
                 , flags: GateFlags::new_interrupt()
                 , ..Default::default()
                 }
        }
    }
}

impl convert::From<ErrorCodeHandler> for Gate<ErrorCodeHandler> {

    /// Creates a new IDT gate pointing at the given handler function.
    ///
    /// The `handler` function must have been created with valid interrupt
    /// calling conventions.
    fn from(handler: ErrorCodeHandler) -> Self {
        unsafe { // trust me on this, `mem::transmute()` is glorious black magic
                let (low, mid, high): (u16, u16, u32) = mem::transmute(handler);

            Gate { offset_lower: low
                 , offset_mid: mid
                 , offset_upper: high
                 , selector: segment::Selector::from_cs()
                 , flags: GateFlags::new_interrupt()
                 , ..Default::default()
                 }
        }
    }
}

impl convert::From<*const u8> for Gate {

    /// Creates a new IDT gate pointing at the given handler function.
    ///
    /// The `handler` function must have been created with valid interrupt
    /// calling conventions.
    ///
    /// This should probably not be used, if it can possibly be avoided.
    //  TODO: it would be really nice if we didn't need this any more.
    //        after the Revolution, once handlers are created in Rust-land with
    //        naked functions...
    fn from(handler: *const u8) -> Self {
        unsafe {
            let (low, mid, high): (u16, u16, u32) = mem::transmute(handler);

            Gate { offset_lower: low
                 , offset_mid: mid
                 , offset_upper: high
                 , selector: segment::Selector::from_cs()
                 , flags: GateFlags::new_interrupt()
                 , ..Default::default()
                 }
        }
    }
}
