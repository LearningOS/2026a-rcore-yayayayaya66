//! Process management syscalls
use crate::mm::{PTEFlags, PageTable, VirtAddr};
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_user_token, syscall_count, user_mapping};
use crate::timer::get_time_us;

// Translate one user byte without dereferencing an invalid or privileged mapping.
fn user_byte(addr: usize, access: PTEFlags) -> Option<&'static mut u8> {
    if addr >= (1usize << 38) {
        return None;
    }
    let va = VirtAddr::from(addr);
    let pte = PageTable::from_token(current_user_token()).translate(va.floor())?;
    if !pte.is_valid() || !pte.flags().contains(PTEFlags::U | access) {
        return None;
    }
    Some(&mut pte.ppn().get_bytes_array()[va.page_offset()])
}

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let addr = ts as usize;
    let us = get_time_us();
    let time = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let bytes = unsafe {
        core::slice::from_raw_parts(
            &time as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };
    for i in 0..bytes.len() {
        if addr
            .checked_add(i)
            .and_then(|a| user_byte(a, PTEFlags::W))
            .is_none()
        {
            return -1;
        }
    }
    for (i, byte) in bytes.iter().enumerate() {
        *user_byte(addr + i, PTEFlags::W).unwrap() = *byte;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    match trace_request {
        0 => user_byte(id, PTEFlags::R).map_or(-1, |b| *b as isize),
        1 => match user_byte(id, PTEFlags::W) {
            Some(b) => {
                *b = data as u8;
                0
            }
            None => -1,
        },
        2 => syscall_count(id),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    user_mapping(start, len, Some(prot))
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    user_mapping(start, len, None)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
