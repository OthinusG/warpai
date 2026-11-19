//! Reproducible native stress and process-overhead measurements for the guardian registry.
use super::*;
#[cfg(target_os = "macos")]
use std::mem::{size_of, MaybeUninit};

fn project_fence(boot: &str) -> ManagedFence {
    ManagedFence {
        service_id: Uuid::new_v4().to_string(),
        service_boot_id: boot.to_owned(),
        connection_id: Uuid::new_v4().to_string(),
        project_id: Uuid::new_v4().to_string(),
    }
}

fn launch_fixture(
    terminals: &Terminals,
    projects: &tasks::Projects,
    root: &Path,
    fence: &ManagedFence,
    command: &str,
) -> TerminalState {
    terminals
        .launch(
            TerminalLaunch {
                fence: Some(fence.clone()),
                session_id: Uuid::new_v4().to_string(),
                executable: "/bin/sh".into(),
                arguments: vec!["-c".into(), command.into()],
                columns: 120,
                rows: 40,
                guarded: true,
                session_kind: Some(TerminalSessionKind::Command as i32),
                ..Default::default()
            },
            fence,
            root,
            projects,
            false,
            false,
            |_, _| Ok(()),
        )
        .unwrap()
}

fn request(fence: &ManagedFence, state: &TerminalState, action: TerminalAction) -> TerminalControl {
    TerminalControl {
        fence: Some(fence.clone()),
        session_id: state.session_id.clone(),
        run_id: state.run_id.clone(),
        action: action as i32,
        attachment_id: state.attachment_id.clone(),
        attachment_generation: state.attachment_generation,
        ..Default::default()
    }
}

#[track_caller]
fn poll_read(
    terminals: &Terminals,
    fence: &ManagedFence,
    state: &TerminalState,
    predicate: impl Fn(&TerminalState) -> bool,
) -> TerminalState {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let reply = terminals
            .control(
                request(fence, state, TerminalAction::TerminalRead),
                fence,
                true,
            )
            .unwrap();
        if predicate(&reply) {
            return reply;
        }
        assert!(
            Instant::now() < deadline,
            "Guarded fixture exceeded its IO deadline"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn native_noisy_and_blocked_projects_leave_a_quiet_writer_and_control_available() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let other = root.join("other");
    std::fs::create_dir(&other).unwrap();
    let projects = tasks::Projects::new(&root);
    let terminals = Terminals::default();
    let boot = Uuid::new_v4().to_string();
    let first_fence = project_fence(&boot);
    let second_fence = project_fence(&boot);
    let noisy = launch_fixture(
        &terminals,
        &projects,
        &root,
        &first_fence,
        "exec /usr/bin/yes noisy",
    );
    let blocked = launch_fixture(
        &terminals,
        &projects,
        &root,
        &first_fence,
        "stty raw -echo; printf 'BLOCKED_READY'; exec sleep 30",
    );
    let quiet = launch_fixture(
        &terminals, &projects, &other, &second_fence,
        "stty -echo; printf 'QUIET_READY'; while IFS= read -r line; do printf '\\r\\nPROCESSED=%s\\r\\n' \"$line\"; done",
    );
    poll_read(&terminals, &first_fence, &blocked, |reply| {
        reply
            .output
            .windows(13)
            .any(|bytes| bytes == b"BLOCKED_READY")
    });
    poll_read(&terminals, &second_fence, &quiet, |reply| {
        reply
            .output
            .windows(11)
            .any(|bytes| bytes == b"QUIET_READY")
    });
    poll_read(&terminals, &first_fence, &noisy, |reply| {
        reply.output_truncated
    });
    for sequence in 1..=16 {
        let mut input = request(&first_fence, &blocked, TerminalAction::TerminalInput);
        input.input_sequence = sequence;
        input.input = vec![b'x'; 4096];
        if terminals.control(input, &first_fence, true).is_err() {
            break;
        }
    }

    let start = Instant::now();
    let marker = b"PROCESSED=quiet-round-trip";
    let mut write = request(&second_fence, &quiet, TerminalAction::TerminalInput);
    write.input_sequence = 1;
    write.input = b"quiet-round-trip\r".to_vec();
    terminals.control(write, &second_fence, true).unwrap();
    poll_read(&terminals, &second_fence, &quiet, |reply| {
        reply
            .output
            .windows(marker.len())
            .any(|bytes| bytes == marker)
    });
    println!(
        "Mixed native workload quiet round trip: {} ms",
        start.elapsed().as_millis()
    );

    let start = Instant::now();
    let stopped = terminals
        .stop_guarded(Uuid::new_v4(), Some(&first_fence.project_id), None)
        .unwrap();
    assert_eq!(stopped.len(), 2);
    assert!(stopped.iter().all(|run| run.stop_requested));
    for session in [&noisy, &blocked] {
        poll_read(&terminals, &first_fence, session, |reply| {
            reply.processes_active == Some(false) && reply.output_closed
        });
    }
    assert_eq!(
        terminals
            .control(
                request(&second_fence, &quiet, TerminalAction::TerminalRead),
                &second_fence,
                true
            )
            .unwrap()
            .processes_active,
        Some(true)
    );
    println!(
        "Noisy and blocked project stopped with native exit proof: {} ms",
        start.elapsed().as_millis()
    );
}

#[derive(Debug)]
struct ProcessMeasurement {
    rss_bytes: u64,
    peak_rss_bytes: u64,
    threads: u64,
    cpu_microseconds: u64,
    voluntary_switches: u64,
    involuntary_switches: u64,
}

fn measure_process() -> ProcessMeasurement {
    let mut usage = unsafe { std::mem::zeroed::<libc::rusage>() };
    assert_eq!(unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) }, 0);
    #[cfg(target_os = "macos")]
    let (rss_bytes, threads, peak_rss_bytes) = {
        let mut info = MaybeUninit::<libc::proc_taskinfo>::uninit();
        let length = size_of::<libc::proc_taskinfo>() as i32;
        assert_eq!(
            unsafe {
                libc::proc_pidinfo(
                    libc::getpid(),
                    libc::PROC_PIDTASKINFO,
                    0,
                    info.as_mut_ptr().cast(),
                    length,
                )
            },
            length
        );
        let info = unsafe { info.assume_init() };
        (
            info.pti_resident_size,
            info.pti_threadnum as u64,
            usage.ru_maxrss as u64,
        )
    };
    #[cfg(target_os = "linux")]
    let (rss_bytes, threads, peak_rss_bytes) = {
        let info = std::fs::read_to_string("/proc/self/status").unwrap();
        let field = |name: &str| {
            info.lines()
                .find_map(|line| {
                    line.strip_prefix(name)
                        .and_then(|value| value.split_whitespace().next())
                })
                .unwrap()
                .parse::<u64>()
                .unwrap()
        };
        (
            field("VmRSS:") * 1024,
            field("Threads:"),
            usage.ru_maxrss as u64 * 1024,
        )
    };
    let time = |value: libc::timeval| value.tv_sec as u64 * 1_000_000 + value.tv_usec as u64;
    ProcessMeasurement {
        rss_bytes,
        peak_rss_bytes,
        threads,
        cpu_microseconds: time(usage.ru_utime) + time(usage.ru_stime),
        voluntary_switches: usage.ru_nvcsw as u64,
        involuntary_switches: usage.ru_nivcsw as u64,
    }
}

#[test]
#[ignore = "native resource measurement, run in isolation"]
fn measure_guardian_idle_0_1_8_32() {
    let seconds = std::env::var("WARPAI_GUARDIAN_SAMPLE_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| (1..=120).contains(seconds))
        .unwrap_or(60);
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let projects = tasks::Projects::new(&root);
    let terminals = Terminals::default();
    let fence = project_fence(&Uuid::new_v4().to_string());
    let mut sessions = Vec::new();
    println!("sessions,wall_ms,rss_bytes,peak_rss_bytes,threads,cpu_us,voluntary_switches,involuntary_switches");
    for count in [0, 1, 8, 32] {
        while sessions.len() < count {
            sessions.push(launch_fixture(
                &terminals,
                &projects,
                &root,
                &fence,
                "exec /bin/cat",
            ));
        }
        std::thread::sleep(Duration::from_millis(250));
        let before = measure_process();
        let start = Instant::now();
        // Match the daemon's lifecycle maintenance cadence without request traffic.
        for _ in 0..seconds {
            std::thread::sleep(Duration::from_secs(1));
            assert_eq!(terminals.active(), count != 0);
        }
        let after = measure_process();
        println!(
            "{count},{},{},{},{},{},{},{}",
            start.elapsed().as_millis(),
            after.rss_bytes,
            after.peak_rss_bytes,
            after.threads,
            after.cpu_microseconds - before.cpu_microseconds,
            after.voluntary_switches - before.voluntary_switches,
            after.involuntary_switches - before.involuntary_switches
        );
    }
    assert_eq!(
        terminals
            .stop_guarded(Uuid::new_v4(), None, None)
            .unwrap()
            .len(),
        sessions.len()
    );
}
