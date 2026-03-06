# libcamera

Rust 嵌入式摄像头全栈软件框架。同一套代码运行在 Linux（多进程）和 RTOS（多线程）上。

## 功能

| 功能 | 组件 | 说明 |
|------|------|------|
| 实时音视频预览 | LiveService | 主/子码流、多观看者 Fan-out |
| 音频对讲 | TalkService | 半双工/全双工、上下行音频 |
| MP4 录制 | RecordService | 分片策略、断电保护、循环覆盖 |
| 音视频回放 | PlaybackService | MP4 demux、时间线查询、Seek |
| 云存储 | CloudService | 断点续传、指数退避重试、上传队列 |
| 系统升级 | UpgradeService | OTA 下载、ed25519 签名校验、A/B 分区 |
| 配置管理 | ConfigService | 分层合并（出厂/用户/云端）、KV 持久化 |
| 存储管理 | StorageManager | SD 卡检测状态机、容量监控、阈值告警 |
| 网络管理 | NetworkManager | WiFi 连接状态机、指数退避重连 |
| 时间同步 | TimeSyncService | SNTP 客户端、大跳变保护 |
| 远程控制 | ControlGateway | App 命令网关、表驱动分发、会话管理、鉴权 |
| 媒体管线 | MediaCore | Pipeline 调度、双码流分发、Fan-out 帧共享 |

## 架构

```
┌────────────────────────────────────────────────────────────┐
│                     Application Layer                      │
│   linux_node (多进程)    │    rtos_node (多线程)              │
├────────────────────────────────────────────────────────────┤
│                     Platform Layer                         │
│   platform_linux (std)   │    platform_rtos (no_std)       │
│   shm_ring / uds_router │    static_ringbuf / semaphore   │
├────────────────────────────────────────────────────────────┤
│                     Service Layer (all no_std)              │
│   ConfigService │ StorageManager │ NetworkManager           │
│   TimeSyncService │ MediaCore │ LiveService │ TalkService   │
│   RecordService │ PlaybackService │ CloudService            │
│   UpgradeService │ ControlGateway                          │
├────────────────────────────────────────────────────────────┤
│                   Communication Layer (no_std)              │
│   SpscRingBuf │ FanOutPublisher │ TopicRouter              │
│   RequestReplyEngine │ SpinMutex │ InProcessCommBus        │
├────────────────────────────────────────────────────────────┤
│                     Core Layer (no_std)                     │
│   core_types: FixedVec │ FixedString │ CamError │ CtrlMsg   │
│               FrameHeader │ Topic │ ServiceId │ cam_log!    │
│   core_interfaces: CommBus │ Service │ PAL traits           │
└────────────────────────────────────────────────────────────┘
```

## 设计约束

- 业务 crate 全部 `#![no_std]`，Linux 平台层允许 `std`
- Release 二进制 **331KB**（远低于 5MB 预算）
- 三方依赖严格白名单，核心数据结构自实现
- 不引入 tokio / async-std / hyper 等重量级运行时

## 工程结构

```
libcamera/
├── Cargo.toml                  # Workspace root + release profile
├── crates/
│   ├── core_types/             # 基础类型、错误码、消息信封
│   ├── core_interfaces/        # CommBus / Service / PAL trait
│   ├── comm/                   # 通信层
│   │   ├── ring_buffer.rs      #   SPSC RingBuffer (lock-free)
│   │   ├── fan_out.rs          #   Fan-out Publisher + RefCountedSlot
│   │   ├── topic_router.rs     #   Topic 路由器
│   │   ├── request_reply.rs    #   非阻塞 Request/Reply 引擎
│   │   ├── spin_mutex.rs       #   no_std SpinMutex
│   │   └── in_process.rs       #   InProcessCommBus
│   ├── media_core/             #   Pipeline 调度 / 帧分发
│   ├── transport_p2p/          #   P2P 通道抽象
│   ├── service_config/         #   配置管理
│   ├── service_storage/        #   存储管理
│   ├── service_network/        #   网络管理
│   ├── service_time/           #   时间同步 (SNTP)
│   ├── service_live/           #   实时预览
│   ├── service_talk/           #   音频对讲
│   ├── service_record/         #   MP4 录制
│   ├── service_playback/       #   音视频回放
│   ├── service_cloud/          #   云存储上传
│   ├── service_upgrade/        #   OTA 升级
│   ├── service_control/        #   ControlGateway
│   ├── platform_linux/         #   Linux PAL (std)
│   │   ├── pal_fs.rs           #     FileSystem -> std::fs
│   │   ├── pal_timer.rs        #     Timer -> std::time
│   │   ├── shm_ring.rs         #     shm RingBuffer (mmap)
│   │   ├── uds_router.rs       #     UDS TopicRouter
│   │   └── daemon/             #     sys-daemon / infra-daemon
│   └── platform_rtos/          #   RTOS PAL (no_std stub)
├── apps/
│   ├── linux_node/             #   Linux 可执行入口
│   └── rtos_node/              #   RTOS 可执行入口 (no_std)
└── tests/
    └── integration/            #   集成测试
```

## 通信模型

| 层面 | Linux | RTOS | 说明 |
|------|-------|------|------|
| 数据面 | shm RingBuffer | 内存直接访问 | 视频/音频帧，零拷贝 |
| 控制面 | Unix Domain Socket | 函数调用 | CmdXxx Request/Reply |
| 事件面 | Unix Domain Socket | EventGroup | EvtXxx Pub/Sub 广播 |

- **26 个 Topic**：数据面 6 + 控制面 13 + 事件面 7
- **Pub/Sub**：事件广播，一对多
- **Request/Reply**：非阻塞两步式（`send_request` → `poll_reply`）
- **Fan-out SPSC**：1 个生产者写入 N 个独立 RingBuffer，运行时动态增减消费者

## 服务启动顺序

```
Level 0:  ConfigService
Level 1:  NetworkManager, StorageManager
Level 2:  TimeSyncService
Level 3:  MediaCore
Level 4:  LiveService, TalkService, RecordService, PlaybackService
Level 5:  CloudService, UpgradeService
Level 6:  ControlGateway
```

各级通过 `Service::dependencies()` 声明依赖，`AppEntry` 按拓扑排序逐级启动。

## 降级状态机

```
Normal ──→ Degraded ──→ Suspended
  ↑            │             │
  └────────────┴─────────────┘  (恢复条件满足)
```

| 服务 | Degraded 触发 | Suspended 触发 |
|------|-------------|--------------|
| RecordService | 存储满 (循环覆盖) | 存储移除 |
| CloudService | 网络弱 | 网络断开 |
| LiveService | -- | 无活跃连接 |
| PlaybackService | -- | 存储移除 |
| UpgradeService | -- | 网络断开/存储不可用 |
| ControlGateway | 网络断开 | -- |
| TimeSyncService | 连续失败>3次 | -- |

## 构建

```bash
# Debug 构建
cargo build

# Release 构建 (体积优化)
cargo build --release

# 运行 (Linux)
./target/release/linux_node
```

Release profile 使用 `opt-level="z"` + `lto="fat"` + `panic="abort"` + `strip=true` 实现最小体积。

## 运行

```bash
$ ./target/release/linux_node

============================================
  libcamera - Rust Embedded Camera System
  Linux InProcess Mode (single-process)
============================================

[bus] InProcessCommBus created (SpinMutex, 64 topics, 8 subs/topic)
[pal] LinuxFileSystem ready

[startup] Staged service initialization (7 levels)...
  Level 0: ConfigService ... OK
  Level 1: NetworkManager, StorageManager ... OK
  Level 2: TimeSyncService ... OK
  Level 3: MediaCore ... OK
  Level 4: LiveService, TalkService, RecordService, PlaybackService ... OK
  Level 5: CloudService, UpgradeService ... OK
  Level 6: ControlGateway ... OK

[startup] All 12 services initialized and started.

[running] Main loop started (poll interval: 10ms). Press Ctrl+C to stop.
[status] uptime=00:00:05  polls=418  all_services=Normal
```

Ctrl+C / SIGTERM 触发优雅关闭，按逆序停止所有服务。

## 测试

```bash
# 运行所有测试
cargo test --workspace --exclude rtos_node --exclude platform_rtos

# 运行指定 crate 测试
cargo test -p comm
cargo test -p service_config
cargo test -p integration_tests
```

当前测试覆盖：

| 类别 | 数量 | 说明 |
|------|------|------|
| core_types | 15 | FixedVec/FixedString/CtrlMsg/FrameHeader |
| comm | 20 | RingBuffer/FanOut/TopicRouter/RR/InProcessCommBus |
| platform_linux | 6 | PAL/shm/UDS |
| 12 个服务单元测试 | 62 | 状态机/命令处理/降级 |
| 集成测试 | 11 | Phase B/E2E/跨平台 |
| **合计** | **114** | **0 failures** |

## KPI

| 指标 | 目标 | 实际 |
|------|------|------|
| Release 二进制体积 | <= 5MB | **331KB** |
| 自动化测试 | >= 100 | **114 tests** |
| 测试通过率 | 100% | **0 failures** |
| no_std crate 数 | >= 15 | **17 crates** |

## 平台支持

| 平台 | 运行模式 | 状态 |
|------|---------|------|
| Linux x86_64 | 单进程 InProcessCommBus | 可运行 |
| Linux (多进程) | 6 进程 shm+UDS | 框架就绪 |
| RTOS | 多线程 InProcessCommBus | 交叉编译 stub |

## License

Private / Proprietary
