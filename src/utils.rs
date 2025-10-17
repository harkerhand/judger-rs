use nix::libc;
use nix::libc::timeval;

pub(crate) fn get_time_us() -> u64 {
    unsafe {
        let mut tv: timeval = std::mem::zeroed();
        libc::gettimeofday(&mut tv, std::ptr::null_mut());
        (tv.tv_sec * 1000 * 1000 + tv.tv_usec) as u64
    }
}