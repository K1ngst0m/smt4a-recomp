//! the widescreen in overrides/widescreen.c, at the shape Zakuro's "Top
//! screen shape" setting stretches the top screen to, so the one setting
//! does both: the game draws the scene that much wider, Zakuro shows it
//! that much wider.

use std::path::PathBuf;
use std::ptr::{addr_of, addr_of_mut};
use std::time::{Duration, Instant};

use zakuro::{GameOverlay, Memory, OverlayText};

extern "C" {
    static mut smt4a_wide: f32;
    static smt4a_wide_counts: [u32; 4];
    static mut smt4a_wide_seen: [[u32; 3]; 64];
    static mut smt4a_wide_seen_count: u32;
    static mut smt4a_quads: [[u32; 6]; 128];
    static mut smt4a_quad_count: u32;
}

/// the (caller, x, y, w, h, calls) of the 2D quads drawn since the last
/// time, cleared.
pub fn take_quads() -> Vec<(u32, [f32; 4], u32)> {
    // SAFETY: as in set
    unsafe {
        let n = addr_of!(smt4a_quad_count).read() as usize;
        let quads = addr_of!(smt4a_quads).read();
        addr_of_mut!(smt4a_quad_count).write(0);
        quads[..n].iter().map(|q| (q[0], [f32::from_bits(q[1]), f32::from_bits(q[2]), f32::from_bits(q[3]), f32::from_bits(q[4])], q[5])).collect()
    }
}

/// the (caller, aspect, calls) the perspective matrix was given since the
/// last time, cleared.
pub fn take_seen() -> Vec<(u32, f32, u32)> {
    // SAFETY: as in set
    unsafe {
        let n = addr_of!(smt4a_wide_seen_count).read() as usize;
        let seen = addr_of!(smt4a_wide_seen).read();
        addr_of_mut!(smt4a_wide_seen_count).write(0);
        seen[..n].iter().map(|&[lr, bits, calls]| (lr, f32::from_bits(bits), calls)).collect()
    }
}

/// the aspect a shape draws the scene at, as Zakuro's settings or the
/// replay's SMT4A_WIDE name it, none for the console's own.
pub fn aspect(shape: &str) -> Option<f32> {
    match shape {
        // Zakuro's widths for them at a height of 240
        "wide16x9" | "16:9" => Some(16.0 / 9.0),
        "wide21x9" | "21:9" => Some(560.0 / 240.0),
        _ => None,
    }
}

pub fn set(aspect: Option<f32>) {
    // SAFETY: the code reads it on this thread only, between frames this
    // writes it
    unsafe { addr_of_mut!(smt4a_wide).write(aspect.unwrap_or(0.0)) }
}

/// how many times the camera, the perspective matrix and the screen's
/// orthographic matrix were changed, and the full screen quads spread.
pub fn counts() -> [u32; 4] {
    // SAFETY: as in set
    unsafe { addr_of!(smt4a_wide_counts).read() }
}

/// where Zakuro keeps its settings, as its settings.rs finds it.
fn settings_path() -> Option<PathBuf> {
    let var = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty()).map(PathBuf::from);
    let dir = if cfg!(windows) {
        var("APPDATA")
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|home| home.join("Library/Application Support"))
    } else {
        var("XDG_CONFIG_HOME").or_else(|| var("HOME").map(|home| home.join(".config")))
    };
    dir.map(|dir| dir.join("zakuro").join("settings.toml"))
}

/// the top_shape line of Zakuro's settings, "console" when there is none.
fn top_shape() -> String {
    let text = settings_path().and_then(|path| std::fs::read_to_string(path).ok()).unwrap_or_default();
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("top_shape"))
        .filter_map(|rest| rest.trim_start().strip_prefix('='))
        .map(|value| value.trim().trim_matches('"').to_owned())
        .next()
        .unwrap_or_else(|| "console".to_owned())
}

/// follows the setting while the game runs. Zakuro saves the settings as
/// soon as the menu changes them, so reading the file now and then is
/// enough. the 3D follows at the game's next scene, when it sets its camera.
pub struct Follow {
    shape: Option<String>,
    read_at: Instant,
}

impl Follow {
    pub fn new() -> Follow {
        // read at the first frame, once Zakuro has its logger up to say so
        Follow { shape: None, read_at: Instant::now() - Duration::from_secs(1) }
    }

    fn read(&mut self) {
        let shape = top_shape();
        if self.shape.as_ref() != Some(&shape) {
            log::info!("widescreen: top screen shape {shape}, the scene drawn at {:?}", aspect(&shape));
            set(aspect(&shape));
            self.shape = Some(shape);
        }
        self.read_at = Instant::now();
    }
}

impl GameOverlay for Follow {
    fn frame(&mut self, _memory: &mut Memory) -> OverlayText {
        if self.read_at.elapsed() >= Duration::from_secs(1) {
            self.read();
        }
        OverlayText::default()
    }
}
