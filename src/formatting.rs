use crate::Float;
use std::time::Duration;

pub fn fmt_si(val: u64) -> String {
    match val as Float {
        x if x < 1e3 => format!("{:.2}",   x),
        x if x < 1e6 => format!("{:.2} k", x / 1e3),
        x if x < 1e9 => format!("{:.2} M", x / 1e6),
        x            => format!("{:.2} G", x / 1e9),
    }
}

pub fn fmt_elapsed(dt: Duration) -> String {
    match dt.as_micros() as Float {
        x if x < 1e6    => format!("{:.1} ms", x / 1e3),
        x if x < 60e6   => format!("{:.1} s",  x / 1e6),
        x if x < 3600e6 => format!("{:.1} m",  x / 60e6),
        x               => format!("{:.1} h",  x / 3600e6),
    }
}
