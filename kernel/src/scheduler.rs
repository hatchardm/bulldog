// File: kernel/src/scheduler.rs

use log::info;

use crate::process::{
    current_pid,
    set_current_pid,
    mark_running,
    set_process_state,
    ProcessState,
    Pid,
    PROCESS_TABLE,
};

/// Return the first READY process found in the process table.
pub fn next_ready_pid() -> Option<Pid> {
    let table = PROCESS_TABLE.lock();

    table
        .iter()
        .find(|p| p.state == ProcessState::Ready)
        .map(|p| p.pid)
}

/// Simple cooperative scheduler.
///
/// Current implementation:
/// - Finds a READY process
/// - Moves current RUNNING process back to READY
/// - Marks next process RUNNING
/// - Updates CURRENT_PID
///
/// No context switching is performed yet.
pub fn schedule() -> Option<Pid> {
    let current = current_pid();

    let next = next_ready_pid()?;

    if current != 0 && current != next {
        set_process_state(current, ProcessState::Ready);
    }

    mark_running(next);
    set_current_pid(next);

    info!(
        "[SCHED] pid={} -> pid={}",
        current,
        next
    );

    Some(next)
}

/// Future scheduler entry point from timer ticks.
pub fn scheduler_tick() {
    let ticks = crate::time::get_ticks();

    if ticks % 1000 == 0 {
        log::info!(
            "[SCHED] tick={} current={}",
            ticks,
            crate::process::current_pid()
        );
    }
}

/// Dump scheduler state to the serial log.
pub fn dump_scheduler_state() {
    let current = current_pid();

    info!("=== SCHEDULER ===");
    info!("current_pid={}", current);

    let table = PROCESS_TABLE.lock();

    for proc in table.iter() {
        info!(
            "pid={} state={:?} name={}",
            proc.pid,
            proc.state,
            proc.name
        );
    }

    info!("=================");
}

