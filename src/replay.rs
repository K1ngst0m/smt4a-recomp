//! plays a recording back headless on the code linked in, the way Zakuro's
//! boot example does for a library, so a session's saves can be compared
//! against the ones the recorded run wrote.
//!
//!     ZAKURO_DATA=dir ZAKURO_REPLAY=inputs.txt smt4a replay <rom> <frames>
//!
//! the console is set up as boot sets it up, so a recording replays the same
//! on both: same name, console id, language and renderer.

use zakuro_core::{loader, Config, FrameOutcome};

pub fn main(args: &[String], linked: zakuro::Linked) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,cranelift_jit=warn")).init();
    let (Some(rom), Some(frames)) = (args.first(), args.get(1).and_then(|f| f.parse::<u64>().ok())) else {
        eprintln!("usage: smt4a replay <rom> <frames>, with ZAKURO_REPLAY and ZAKURO_DATA set");
        std::process::exit(2);
    };
    let Some(file) = std::env::var_os("ZAKURO_REPLAY") else {
        eprintln!("ZAKURO_REPLAY is not set");
        std::process::exit(2);
    };
    let mut replay = zakuro_core::replay::Replay::open(std::path::Path::new(&file)).unwrap_or_else(|error| {
        eprintln!("could not read {}: {error}", file.to_string_lossy());
        std::process::exit(1);
    });
    let config = Config {
        linked: Some(linked),
        data_dir: std::env::var_os("ZAKURO_DATA").map(Into::into),
        clock: Some(replay.clock()),
        ..Config::default()
    };
    let mut system = loader::load(rom, config).unwrap_or_else(|error| {
        eprintln!("failed to load {rom}: {error}");
        std::process::exit(1);
    });
    // what gets typed when the game opens the software keyboard, as boot does
    let typed = std::env::var("ZAKURO_KEYBOARD").unwrap_or_else(|_| "Zakuro".to_owned());

    let start = std::time::Instant::now();
    let mut outcome = FrameOutcome::Completed;
    let mut executed = 0;
    for frame in 0..frames {
        system.set_input(replay.input(frame));
        let faults = system.memory.fault_summary().len();
        outcome = system.run_frame();
        executed = frame + 1;
        if system.memory.fault_summary().len() != faults {
            println!("frame {executed}: touched an unmapped page");
        }
        if let Some(request) = system.keyboard_request() {
            let confirm = request.buttons.len() - 1;
            system.answer_keyboard(&typed, confirm);
        }
        if outcome != FrameOutcome::Completed {
            break;
        }
    }
    println!("\n--- result ---");
    println!("outcome:       {outcome:?} after {executed} frames in {:.2?}", start.elapsed());
    println!("unmapped pages touched: {}", system.memory.fault_summary().len());
    println!("{}", system.status_line());
    if !system.fatal_errors.is_empty() {
        println!("fatal errors: {}", system.fatal_errors.join("; "));
    }
}
