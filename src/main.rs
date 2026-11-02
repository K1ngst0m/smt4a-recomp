//! Shin Megami Tensei IV: Apocalypse (USA), recompiled ahead of time by
//! 3dsrecomp and run by Zakuro. it still reads the game for everything but
//! the code, so it takes the path to your own copy of it the way Zakuro does.
//!
//!     smt4a [zakuro's options] <rom>
//!     smt4a replay <rom> <frames>     plays a recording back headless, see replay.rs

mod replay;
mod widescreen;

use std::ffi::c_void;
use std::ptr::addr_of;

extern "C" {
    static recomp_abi: u32;
    static recomp_generation: u32;
    static recomp_entry_count: u32;
    static recomp_entries: u8;
    static recomp_module_count: u32;
    static recomp_modules: u8;
    static recomp_origins: u8;
}

const TITLE_ID: u64 = 0x0004_0000_0019_A200;

/// the code build.rs links in, with the tables 3dsrecomp wrote beside it.
fn linked() -> zakuro::Linked {
    // SAFETY: the recomp_ symbols of the code generated for this title
    unsafe {
        zakuro::Linked::new(
            TITLE_ID,
            recomp_abi,
            addr_of!(recomp_entries) as *const c_void,
            recomp_entry_count,
            addr_of!(recomp_modules) as *const c_void,
            recomp_module_count,
        )
        // so that functions a mod or an update changed run in the interpreter
        .with_origins(recomp_generation, addr_of!(recomp_origins) as *const c_void)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("replay") {
        replay::main(&args[1..], linked());
        return;
    }
    zakuro::run_with(Some(linked()), Some(Box::new(widescreen::Follow::new())));
}
