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

const HEAP_SIZE: usize = 1 << 15;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[rtic::app(device = lm3s6965, dispatchers = [GPIOA, GPIOB, GPIOC])]
mod app {
    use crate::{HEAP, HEAP_SIZE};
    use alloc::string::String;
    use alloc::vec::Vec;
    use cortex_m_semihosting::debug::EXIT_SUCCESS;
    use cortex_m_semihosting::heprintln;
    use fandango::generation::Generated;
    use fandango::tuple_list::tuple_list;
    use fandango::visitor::write::WriteVisitor;
    use fandango::visitor::Visitor;
    use fandango_eval::operators::DepthLimiter;
    use fandango_eval::xml;
    use rand::SeedableRng;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    #[allow(unsafe_code)]
    #[allow(unreachable_code)]
    #[allow(static_mut_refs)]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        {
            use core::mem::MaybeUninit;
            static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
            unsafe { HEAP.init(HEAP_MEM.as_mut_ptr() as usize, HEAP_SIZE) }
        }

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        let mut rng = rand::rngs::StdRng::from_seed([0u8; 32]);
        let limiter = DepthLimiter::new::<xml::Type<'static>>(100);
        let mut generators = tuple_list!(limiter);
        for _ in 0..10_000 {
            let mut start = xml::nonterminal_start::generate(&mut rng, &mut generators, 0);
            let _ = xml::ConstraintFixer::corrected(&mut rng, &mut ())
                .visit(&mut start, 0)
                .unwrap();

            let serialized = String::from_utf8(
                WriteVisitor::new(Vec::new())
                    .visit(&mut start, 0)
                    .unwrap()
                    .continue_value()
                    .unwrap()
                    .output(),
            )
            .unwrap();

            heprintln!("{}", serialized);
        }

        cortex_m_semihosting::debug::exit(EXIT_SUCCESS);

        loop {
            cortex_m::asm::wfi();
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
