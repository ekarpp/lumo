use crate::Float;

pub fn fmt_si(val: u64) -> String {
    let val = val as Float;

    if val > 1e9 {
        format!("{:.2} G", val / 1e9)
    } else if val > 1e6 {
        format!("{:.2} M", val / 1e6)
    } else if val > 1e3 {
        format!("{:.2} k", val / 1e3)
    } else {
        format!("{:.2}", val)
    }
}

pub fn fmt_ms(ms: Float, accurate: bool) -> String {
    let sec = ms / 1e3;
    if sec <= 60.0 {
        if accurate {
            format!("{:.3} s", sec)
        } else {
            format!("{:.0} s", sec)
        }
    } else if sec <= 60.0 * 60.0 {
        format!("{:.1} m", sec / 60.0)
    } else {
        format!("{:.1} h", sec / 3600.0)
    }
}
