use std::path::PathBuf;
use std::time::Duration;

pub fn path_to_string(path: &PathBuf, default: &str) -> String {
    match path.to_str() {
        Some(s) => s.to_string(),
        None => default.to_string(),
    }
}

pub fn format_duration(du: Duration) -> String {
    let ms = du.as_millis();
    if ms < 1000 {
        return format!("{}ms", ms);
    }
    let s = du.as_secs();
    if s < 60 {
        return format!("{}s", s);
    }
    let m = s / 60;
    let s = s % 60;
    if m < 60 {
        return format!("{}m{}s", m, s);
    }
    let h = m / 60;
    let m = m % 60;
    return format!("{}h{}m{}s", h, m, s);
}
