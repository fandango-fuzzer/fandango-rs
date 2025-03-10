#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use cortex_m_semihosting::{
    debug::{exit, EXIT_FAILURE},
    heprintln,
};
use panic_semihosting as _;

use core::alloc::Layout;
use embedded_alloc::Heap;
use fandango::Fandango;

const HEAP_SIZE: usize = 1 << 14;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[derive(Fandango)]
#[grammar = "../tests/grammars/xml.fan"]
pub struct Xml;

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA, GPIOB, GPIOC])]
mod app {
    use super::nonterminal_start;
    use crate::{HEAP, HEAP_SIZE};
    use alloc::string::String;
    use alloc::vec::Vec;
    use cortex_m_semihosting::heprintln;
    use fandango::generation::DefaultGenerated;
    use fandango::visitor::write::WriteVisitor;
    use fandango::visitor::Visitor;
    use rand::SeedableRng;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    #[allow(unsafe_code)]
    #[allow(unreachable_code)]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        {
            use core::mem::MaybeUninit;
            static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
            unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
        }

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let mut rng = rand::rngs::StdRng::from_seed([0u8; 32]);
        loop {
            let mut start = nonterminal_start::generate_default(&mut rng, &mut ());

            let serialized = String::from_utf8(
                WriteVisitor::caching(Vec::new())
                    .visit(&mut start, 0)
                    .unwrap()
                    .continue_value()
                    .unwrap()
                    .output(),
            )
            .unwrap();

            heprintln!("{}", serialized);
        }
    }
}

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    heprintln!("Whoops! Ran out of memory while executing.");
    exit(EXIT_FAILURE);

    loop {
        cortex_m::asm::nop();
    }
}
