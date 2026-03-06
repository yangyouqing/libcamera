use platform_linux::pal_fs::LinuxFileSystem;
use platform_linux::pal_timer::LinuxTimer;
use core_interfaces::{FileSystem, Timer};
use std::fs;

// ══════════════════════════════════════════════
// Step 16: PAL tests
// ══════════════════════════════════════════════

#[test]
fn fs_write_read_roundtrip() {
    let fs_impl = LinuxFileSystem::new();
    let path = "/tmp/cam_test_fs_roundtrip.txt";

    let data = b"hello camera";
    fs_impl.write_file(path, data).unwrap();

    let mut buf = [0u8; 64];
    let n = fs_impl.read_file(path, &mut buf).unwrap();
    assert_eq!(&buf[..n], data);

    assert!(fs_impl.file_exists(path));
    assert_eq!(fs_impl.file_size(path).unwrap(), data.len() as u64);

    fs_impl.remove_file(path).unwrap();
    assert!(!fs_impl.file_exists(path));
}

#[test]
fn fs_create_dir_and_list() {
    let fs_impl = LinuxFileSystem::new();
    let dir = "/tmp/cam_test_dir";
    let _ = fs::remove_dir_all(dir);
    fs_impl.create_dir(dir).unwrap();

    fs_impl.write_file(&format!("{}/a.txt", dir), b"a").unwrap();
    fs_impl.write_file(&format!("{}/b.txt", dir), b"b").unwrap();

    let mut entries = [0u8; 512];
    let n = fs_impl.list_dir(dir, &mut entries, 10).unwrap();
    assert!(n > 0);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn timer_monotonic_increasing() {
    let timer = LinuxTimer::new();
    let t1 = timer.monotonic_ms();
    timer.sleep_ms(10);
    let t2 = timer.monotonic_ms();
    assert!(t2 > t1, "monotonic clock must increase: {} vs {}", t1, t2);
}

// ══════════════════════════════════════════════
// Step 18: shm + UDS tests
// ══════════════════════════════════════════════

#[test]
fn shm_ring_push_pop() {
    use platform_linux::shm_ring::ShmRingBuf;

    let ring = ShmRingBuf::create(128, 8).unwrap();
    assert!(ring.is_empty());

    let data = vec![0xABu8; 128];
    assert!(ring.push(&data));
    assert!(!ring.is_empty());

    let mut out = vec![0u8; 128];
    let n = ring.pop(&mut out).unwrap();
    assert_eq!(n, 128);
    assert_eq!(out[0], 0xAB);
}

#[test]
fn shm_ring_cross_process() {
    use platform_linux::shm_ring::ShmRingBuf;
    use std::os::unix::io::{FromRawFd, IntoRawFd};

    let ring = ShmRingBuf::create(64, 4).unwrap();
    let fd = ring.fd();

    // Write from parent
    ring.push(&[0x42u8; 64]);

    // Open from the same fd (simulates child process mapping)
    let ring2 = ShmRingBuf::from_fd(fd, 64, 4).unwrap();
    let mut out = [0u8; 64];
    let n = ring2.pop(&mut out).unwrap();
    assert_eq!(n, 64);
    assert_eq!(out[0], 0x42);

    // Note: don't let ring2 close the fd since ring owns it.
    // In real usage, child gets a dup'd fd via SCM_RIGHTS.
    std::mem::forget(ring2);
}

#[test]
fn uds_router_basic() {
    use platform_linux::uds_router::{UdsTopicRouter, UdsClient2};
    use core_types::Topic;

    let socket_path = "/tmp/cam_test_uds.sock";
    let _ = std::fs::remove_file(socket_path);

    let mut router = UdsTopicRouter::new(socket_path).unwrap();
    let mut client = UdsClient2::connect(socket_path).unwrap();

    // Accept the client connection
    assert_eq!(router.accept_connections(), 1);
    assert_eq!(router.client_count(), 1);

    // Subscribe client to a topic
    router.subscribe_client(0, Topic::EvtConfigChanged);

    // Route a message
    let msg = b"test_event_data";
    let sent = router.route(Topic::EvtConfigChanged, msg);
    assert_eq!(sent, 1);

    // Client receives it
    let mut buf = [0u8; 256];
    let n = client.recv(&mut buf).unwrap();
    assert_eq!(&buf[..n], msg);

    let _ = std::fs::remove_file(socket_path);
}
