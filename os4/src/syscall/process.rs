use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{MapPermission, VirtAddr};
use crate::task::{
    current_m_map, current_m_unmap, current_start_time, current_syscall_info,
    exit_current_and_run_next, suspend_current_and_run_next, translate, TaskStatus,
};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    match translate(VirtAddr(ts as usize)) {
        Some(pa) => {
            unsafe {
                /// updating pa in kernel means updating the va in user
                let p_ts = pa.0 as *mut TimeVal;
                *p_ts = TimeVal {
                    sec: us / 1_000_000,
                    usec: us % 1_000_000,
                }
            };
            0
        }
        None => -1,
    }
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    let va = VirtAddr(start);
    if !va.aligned() {
        return -1;
    };
    if !((port > 0) & (port < 8)) {
        return -1;
    }
    let perm = MapPermission::from_bits(((port << 1) + 16) as u8).unwrap();
    current_m_map(va, len, perm)
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    let va = VirtAddr(start);
    if !va.aligned() {
        return -1;
    };
    current_m_unmap(va, len)
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    match translate(VirtAddr(ti as usize)) {
        Some(pa) => {
            unsafe {
                let ti = pa.0 as *mut TaskInfo;
                *ti = TaskInfo {
                    status: TaskStatus::Running,
                    syscall_times: *current_syscall_info(),
                    time: (get_time_us() - current_start_time()) / 1_000,
                }
            };
            0
        }
        None => -1,
    }
}
