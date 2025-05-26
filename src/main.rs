use std::{
    collections::HashSet,
    ffi::CString,
    fs::File,
    io::{BufRead, BufReader},
    mem,
};

use libc::{c_char, statvfs, statvfs as Statvfs};

#[derive(Debug)]
struct MountInfo {
    source: String,
    mount_point: String,
    #[allow(dead_code)]
    fs_type: String,
    #[allow(dead_code)]
    options: String,
}

fn main() -> std::io::Result<()> {
    let file = File::open("/proc/mounts")?;
    let reader = BufReader::new(file);

    let ignored_fs_types = [
        "proc",
        "sysfs",
        "tmpfs",
        "devtmpfs",
        "devpts",
        "cgroup",
        "cgroup2",
        "overlay",
        "mqueue",
        "debugfs",
        "securityfs",
        "pstore",
        "bpf",
        "fusectl",
        "configfs",
        "hugetlbfs",
        "tracefs",
        "autofs",
        "binfmt_misc",
        "rpc_pipefs",
        "efivarfs",
        "fuse.portal",
        "squashfs",
    ];

    let mut seen_source = HashSet::new();

    header();

    for line in reader.lines() {
        let line = line?;

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 4 {
            let source = parts[0].to_string();
            let mount_point = parts[1].replace("\\040", " ");
            let fs_type = parts[2].to_string();
            let options = parts[3].to_string();

            if ignored_fs_types.contains(&fs_type.as_str()) {
                continue;
            }

            if !seen_source.insert(source.clone()) {
                continue;
            }

            let mount = MountInfo {
                source,
                mount_point,
                fs_type,
                options,
            };

            if let Some((total, free)) = get_usage(&mount.mount_point) {
                let used = total - free;
                let pct_used = (used as f64 / total as f64) * 100.0;
                println!(
                    "{:35} {:>6} {:>6} {:>6} {:>6.1}% {} {:<}",
                    mount.source,
                    to_gib(total),
                    to_gib(used),
                    to_gib(free),
                    pct_used,
                    usage_bar(pct_used),
                    mount.mount_point
                );
            }
        }
    }

    Ok(())
}

fn get_usage(path: &str) -> Option<(u64, u64)> {
    let c_path = CString::new(path).ok()?;
    let mut stat: Statvfs = unsafe { mem::zeroed() };

    let ret = unsafe { statvfs(c_path.as_ptr() as *const c_char, &mut stat) };

    if ret == 0 {
        let total = stat.f_blocks as u64 * stat.f_frsize as u64;
        let free = stat.f_bavail as u64 * stat.f_frsize as u64;
        Some((total, free))
    } else {
        None
    }
}

fn usage_bar(pct: f64) -> String {
    const RESET: &str = "\x1b[0m";
    const FG_GREEN: &str = "\x1b[32m";
    const FG_YELLOW: &str = "\x1b[33m";
    const FG_RED: &str = "\x1b[31m";

    let total_blocks = 20;
    let filled = ((pct / 100.0) * total_blocks as f64).round() as usize;

    let color = match pct {
        p if p < 75.0 => FG_GREEN,
        p if p < 85.0 => FG_YELLOW,
        _ => FG_RED,
    };

    format!(
        "{}{:<20}{}",
        color,
        "■".repeat(filled).chars().take(20).collect::<String>(),
        RESET
    )
}

fn to_gib(bytes: u64) -> String {
    format!("{:.1}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
}
fn header() {
    const BOLD: &str = "\x1b[1m";
    const FG_CYAN: &str = "\x1b[36m";
    const RESET: &str = "\x1b[0m";

    println!(
        "{}{}{:35} {:>6} {:>6} {:>6} {:>6} {:<20}{} {:<}{}",
        BOLD,
        FG_CYAN,
        "Filesystem",
        "Size",
        "Used",
        "Avail",
        "Use%",
        "Graph",
        RESET,
        "Mounted on",
        RESET
    );
}
