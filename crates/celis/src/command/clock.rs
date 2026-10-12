//! Times as the terminal shows them, in the local time zone.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The time `at` as `14:02` on the local day of `now`, else as `Oct 9`.
pub(super) fn clock(at: u64, now: u64) -> String {
    let (then, today) = (local(at), local(now));
    match (then.tm_year, then.tm_yday) == (today.tm_year, today.tm_yday) {
        true => format!("{:02}:{:02}", then.tm_hour, then.tm_min),
        false => format!(
            "{} {}",
            MONTHS[then.tm_mon.clamp(0, 11) as usize],
            then.tm_mday
        ),
    }
}

/// The local day of `at`, as `Oct 9`.
pub(super) fn day(at: u64) -> String {
    let tm = local(at);
    format!("{} {}", MONTHS[tm.tm_mon.clamp(0, 11) as usize], tm.tm_mday)
}

/// The time of the local midnight that starts the day of `now`.
pub(super) fn midnight(now: u64) -> u64 {
    let tm = local(now);
    let since = tm.tm_hour * 3_600 + tm.tm_min * 60 + tm.tm_sec;
    now.saturating_sub(u64::try_from(since).unwrap_or(0))
}

/// The local calendar time of `at`.
fn local(at: u64) -> libc::tm {
    let time = libc::time_t::try_from(at).unwrap_or(0);
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&time, &mut tm) };
    tm
}

/// The local day of `at`, as `2026-10-12`.
pub(super) fn date(at: u64) -> String {
    let tm = local(at);
    format!(
        "{}-{:02}-{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday
    )
}
