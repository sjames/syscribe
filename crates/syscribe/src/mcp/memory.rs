//! Memory behaviour of the MCP server (`REQ-TRS-MCP-MEM-000`).
//!
//! The server keeps the whole model in memory and rebuilds it on every reload and
//! guarded write, from worker threads. With glibc's default of many per-thread
//! arenas the freed memory of each rebuild stayed resident and the process grew
//! with every write, so on Linux/glibc the arena count is capped at startup and
//! free memory is handed back after each rebuild. Resident size is reported by
//! the `server_stats` tool and logged on reload.

/// Resident and peak resident memory of this process, in KiB. `None` where the
/// platform offers no cheap reading (only Linux does).
pub fn usage_kb() -> Option<(u64, u64)> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let field = |name: &str| -> Option<u64> {
            status.lines().find(|l| l.starts_with(name))?.split_whitespace().nth(1)?.parse().ok()
        };
        Some((field("VmRSS:")?, field("VmHWM:")?))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Cap glibc's malloc arenas. Call once, before the worker threads start.
pub fn tune_allocator() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    // SAFETY: `mallopt` only sets an allocator parameter and is called before threads exist.
    unsafe {
        libc::mallopt(libc::M_ARENA_MAX, 2);
    }
}

/// Return freed heap pages to the operating system after a model rebuild.
pub fn release_free_memory() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    // SAFETY: `malloc_trim` is safe to call at any time from any thread.
    unsafe {
        libc::malloc_trim(0);
    }
}
