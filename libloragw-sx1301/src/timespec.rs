use std::ops::Add;
use std::time::{Duration, SystemTime};

use super::wrapper;

pub fn timespec_to_system_time(ts: &wrapper::timespec) -> SystemTime {
    SystemTime::UNIX_EPOCH
        .add(Duration::from_secs(ts.tv_sec as u64) + Duration::from_nanos(ts.tv_nsec as u64))
}

pub fn system_time_to_timespec(st: &SystemTime) -> wrapper::timespec {
    let utc_dur = st.duration_since(SystemTime::UNIX_EPOCH).unwrap();

    #[allow(clippy::needless_update)]
    wrapper::timespec {
        tv_sec: utc_dur.as_secs() as wrapper::time_t,
        tv_nsec: (utc_dur.as_nanos() % 1000000000) as std::os::raw::c_long,
        ..Default::default()
    }
}


pub fn duration_to_timespec(d: &Duration) -> wrapper::timespec {
    #[allow(clippy::needless_update)]
    wrapper::timespec {
        tv_sec: d.as_secs() as wrapper::time_t,
        tv_nsec: (d.as_nanos() % 1000000000) as std::os::raw::c_long,
        ..Default::default()
    }
}

pub fn timespec_to_duration(ts: &wrapper::timespec) -> Duration {
    Duration::from_secs(ts.tv_sec as u64) + Duration::from_nanos(ts.tv_nsec as u64)
}
