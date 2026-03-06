use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use core_types::ServiceId;
use core_interfaces::Service;
use comm::in_process::InProcessCommBus;
use platform_linux::daemon::app_entry::STARTUP_ORDER;
use platform_linux::pal_fs::LinuxFileSystem;

use service_config::ConfigService;
use service_storage::StorageManager;
use service_network::NetworkManager;
use service_time::TimeSyncService;
use media_core::MediaCoreService;
use service_live::LiveService;
use service_talk::TalkService;
use service_record::RecordService;
use service_playback::PlaybackService;
use service_cloud::CloudService;
use service_upgrade::UpgradeService;
use service_control::ControlGateway;

static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn sig_handler(_signum: libc::c_int) {
    RUNNING.store(false, Ordering::SeqCst);
}

fn install_signal_handlers() {
    unsafe {
        let mut sa: libc::sigaction = core::mem::zeroed();
        sa.sa_sigaction = sig_handler as usize;
        sa.sa_flags = 0;
        libc::sigemptyset(&mut sa.sa_mask);
        libc::sigaction(libc::SIGINT, &sa, core::ptr::null_mut());
        libc::sigaction(libc::SIGTERM, &sa, core::ptr::null_mut());
    }
}

fn sid_name(sid: ServiceId) -> &'static str {
    match sid {
        ServiceId::Config => "ConfigService",
        ServiceId::Network => "NetworkManager",
        ServiceId::Storage => "StorageManager",
        ServiceId::TimeSync => "TimeSyncService",
        ServiceId::MediaCore => "MediaCore",
        ServiceId::Live => "LiveService",
        ServiceId::Talk => "TalkService",
        ServiceId::Record => "RecordService",
        ServiceId::Playback => "PlaybackService",
        ServiceId::Cloud => "CloudService",
        ServiceId::Upgrade => "UpgradeService",
        ServiceId::ControlGateway => "ControlGateway",
    }
}

macro_rules! report_health {
    ($($name:expr => $svc:expr),+ $(,)?) => {{
        let mut all_ok = true;
        $(
        {
            let h = $svc.health();
            if h.state != core_types::ServiceState::Normal {
                if all_ok { print!("alerts: "); all_ok = false; }
                print!("{}={:?} ", $name, h.state);
            }
        }
        )+
        if all_ok { print!("all_services=Normal"); }
    }};
}

fn main() {
    install_signal_handlers();

    println!("============================================");
    println!("  libcamera - Rust Embedded Camera System");
    println!("  Linux InProcess Mode (single-process)");
    println!("============================================");
    println!();

    let bus: &'static InProcessCommBus = Box::leak(Box::new(InProcessCommBus::new()));
    println!("[bus] InProcessCommBus created (SpinMutex, 64 topics, 8 subs/topic)");

    let fs = LinuxFileSystem::new();
    println!("[pal] LinuxFileSystem ready");

    let mut config = ConfigService::new();
    config.set_filesystem(&fs);
    let mut storage = StorageManager::new();
    let mut network = NetworkManager::new();
    let mut time_svc = TimeSyncService::new();
    let mut media_core = MediaCoreService::new();
    let mut live = LiveService::new();
    let mut talk = TalkService::new();
    let mut record = RecordService::new();
    let mut playback = PlaybackService::new();
    let mut cloud = CloudService::new();
    let mut upgrade = UpgradeService::new();
    let mut control = ControlGateway::new();

    println!();
    println!("[startup] Staged service initialization (7 levels)...");

    for (level_idx, level) in STARTUP_ORDER.iter().enumerate() {
        print!("  Level {}: ", level_idx);
        for (i, &sid) in level.iter().enumerate() {
            if i > 0 { print!(", "); }
            print!("{}", sid_name(sid));
            let result = match sid {
                ServiceId::Config => config.init(bus).and_then(|_| config.start()),
                ServiceId::Network => network.init(bus).and_then(|_| network.start()),
                ServiceId::Storage => storage.init(bus).and_then(|_| storage.start()),
                ServiceId::TimeSync => time_svc.init(bus).and_then(|_| time_svc.start()),
                ServiceId::MediaCore => media_core.init(bus).and_then(|_| media_core.start()),
                ServiceId::Live => live.init(bus).and_then(|_| live.start()),
                ServiceId::Talk => talk.init(bus).and_then(|_| talk.start()),
                ServiceId::Record => record.init(bus).and_then(|_| record.start()),
                ServiceId::Playback => playback.init(bus).and_then(|_| playback.start()),
                ServiceId::Cloud => cloud.init(bus).and_then(|_| cloud.start()),
                ServiceId::Upgrade => upgrade.init(bus).and_then(|_| upgrade.start()),
                ServiceId::ControlGateway => control.init(bus).and_then(|_| control.start()),
            };
            if let Err(e) = result {
                println!(" [FAIL: {:?}]", e);
            }
        }
        println!(" ... OK");
    }

    println!();
    println!("[startup] All 12 services initialized and started.");
    println!();

    println!("--- Service Health Report ---");
    println!("  Config={:?}  Storage={:?}  Network={:?}  TimeSync={:?}",
        config.health().state, storage.health().state,
        network.health().state, time_svc.health().state);
    println!("  MediaCore={:?}  Live={:?}  Talk={:?}  Record={:?}",
        media_core.health().state, live.health().state,
        talk.health().state, record.health().state);
    println!("  Playback={:?}  Cloud={:?}  Upgrade={:?}  Control={:?}",
        playback.health().state, cloud.health().state,
        upgrade.health().state, control.health().state);
    println!("-----------------------------");
    println!();
    println!("[running] Main loop started (poll interval: 10ms). Press Ctrl+C to stop.");
    println!();

    let start = Instant::now();
    let mut poll_count: u64 = 0;
    let mut last_report = Instant::now();
    let report_interval = std::time::Duration::from_secs(5);

    while RUNNING.load(Ordering::Relaxed) {
        let _ = config.poll();
        let _ = storage.poll();
        let _ = network.poll();
        let _ = time_svc.poll();
        let _ = media_core.poll();
        let _ = live.poll();
        let _ = talk.poll();
        let _ = record.poll();
        let _ = playback.poll();
        let _ = cloud.poll();
        let _ = upgrade.poll();
        let _ = control.poll();

        poll_count += 1;

        if last_report.elapsed() >= report_interval {
            let uptime = start.elapsed().as_secs();
            let h = uptime / 3600;
            let m = (uptime % 3600) / 60;
            let s = uptime % 60;
            print!("[status] uptime={:02}:{:02}:{:02}  polls={}  ", h, m, s, poll_count);
            report_health!(
                "Config" => config, "Storage" => storage, "Network" => network,
                "TimeSync" => time_svc, "MediaCore" => media_core, "Live" => live,
                "Talk" => talk, "Record" => record, "Playback" => playback,
                "Cloud" => cloud, "Upgrade" => upgrade, "Control" => control,
            );
            println!();
            last_report = Instant::now();
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!();
    println!("[shutdown] Stopping services...");

    let _ = control.stop();
    let _ = upgrade.stop();
    let _ = cloud.stop();
    let _ = playback.stop();
    let _ = record.stop();
    let _ = talk.stop();
    let _ = live.stop();
    let _ = media_core.stop();
    let _ = time_svc.stop();
    let _ = network.stop();
    let _ = storage.stop();
    let _ = config.stop();

    let uptime = start.elapsed().as_secs();
    println!("[shutdown] All services stopped. Total uptime: {}s, polls: {}", uptime, poll_count);
    println!("[shutdown] Goodbye.");
}
