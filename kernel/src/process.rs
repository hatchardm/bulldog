use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use log::info;

pub type Pid = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Exited,
}

#[derive(Debug)]
pub struct Process {
    pub pid: Pid,
    pub state: ProcessState,

    pub name: String,

    pub entry: u64,
    pub stack_top: u64,

    pub exit_code: Option<u64>,
}

static NEXT_PID: AtomicU64 = AtomicU64::new(1);

pub static PROCESS_TABLE: Mutex<Vec<Process>> =
    Mutex::new(Vec::new());

pub fn allocate_pid() -> Pid {
    NEXT_PID.fetch_add(1, Ordering::SeqCst)
}

pub fn create_process(name: &str) -> Pid {
    let pid = allocate_pid();

    PROCESS_TABLE.lock().push(Process {
        pid,
        state: ProcessState::Ready,
        name: String::from(name),
        entry: 0,
        stack_top: 0,
        exit_code: None,
    });

    info!("PID {} created for {}", pid, name);

    pid
}

pub fn dump_process_table() {
    let table = PROCESS_TABLE.lock();

    info!("=== PROCESS TABLE ===");

    for p in table.iter() {
        info!(
    "pid={} state={:?} exit={:?} entry={:#x} stack={:#x} name={}",
    p.pid,
    p.state,
    p.exit_code,
    p.entry,
    p.stack_top,
    p.name
);
    }

    info!("=====================");
}

pub fn update_process_execution(
    pid: Pid,
    entry: u64,
    stack_top: u64,
) {
    let mut table = PROCESS_TABLE.lock();

    if let Some(proc) = table.iter_mut().find(|p| p.pid == pid) {
        proc.entry = entry;
        proc.stack_top = stack_top;
    }
}

static CURRENT_PID: AtomicU64 = AtomicU64::new(0);

pub fn set_current_pid(pid: Pid) {
    CURRENT_PID.store(pid, Ordering::SeqCst);
}

pub fn current_pid() -> Pid {
    CURRENT_PID.load(Ordering::SeqCst)
}

pub fn set_process_state(
    pid: Pid,
    state: ProcessState,
) {
    let mut table = PROCESS_TABLE.lock();

    if let Some(proc) =
        table.iter_mut().find(|p| p.pid == pid)
    {
        proc.state = state;
    }
}

pub fn mark_running(pid: Pid) {
    set_process_state(pid, ProcessState::Running);
}

pub fn mark_exited(
    pid: Pid,
    exit_code: u64,
) {
    let mut table = PROCESS_TABLE.lock();

    if let Some(proc) =
        table.iter_mut().find(|p| p.pid == pid)
    {
        proc.state = ProcessState::Exited;
        proc.exit_code = Some(exit_code);
    }
}