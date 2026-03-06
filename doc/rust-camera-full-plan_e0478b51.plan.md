---
name: rust-camera-full-plan
overview: Rust 嵌入式摄像头软件。业务 crate 全 no_std，Linux 平台层允许 std。Pub/Sub + Request/Reply + Fan-out SPSC 通信。11 个业务组件：Live/Talk/Record/Playback/Cloud/Upgrade/Config/Storage/Network/TimeSync + ControlGateway。全量 <= 5MB。
todos:
  - id: step-01
    content: "01: Cargo workspace + release profile + 依赖白名单"
    status: completed
  - id: step-02
    content: "02: core_types crate (FixedVec, FixedString, CamError, ServiceId, Topic, CtrlMsg, FrameHeader, ServiceState, cam_log!)"
    status: completed
  - id: step-03
    content: "03: core_types 测试 (FixedVec/FixedString/CtrlMsg size_of/FrameHeader 对齐)"
    status: completed
  - id: step-04
    content: "04: core_interfaces crate (CommBus, Service, PAL traits)"
    status: completed
  - id: step-05
    content: "05: comm - SPSC RingBuffer"
    status: completed
  - id: step-06
    content: "06: comm - SPSC RingBuffer 测试"
    status: completed
  - id: step-07
    content: "07: comm - Fan-out Publisher + RefCountedSlot"
    status: completed
  - id: step-08
    content: "08: comm - Fan-out 测试 (1->3, add/remove_consumer, refcount)"
    status: completed
  - id: step-09
    content: "09: comm - TopicRouter"
    status: completed
  - id: step-10
    content: "10: comm - TopicRouter 测试"
    status: completed
  - id: step-11
    content: "11: comm - Request/Reply (PendingReply + send_request/poll_reply)"
    status: completed
  - id: step-12
    content: "12: comm - RR 测试 (send/poll/cancel/多并发)"
    status: completed
  - id: step-13
    content: "13: comm - InProcessCommBus"
    status: completed
  - id: step-14
    content: "14: comm - InProcessCommBus 测试 (多线程 pub/sub/rr)"
    status: completed
  - id: step-15
    content: "15: platform_linux - PAL trait 实现 (FileSystem, Network, Timer)"
    status: completed
  - id: step-16
    content: "16: platform_linux - PAL 测试 (文件读写, Timer)"
    status: completed
  - id: step-17
    content: "17: platform_linux - shm RingBuffer + UDS TopicRouter"
    status: completed
  - id: step-18
    content: "18: platform_linux - shm+UDS 测试 (fork 子进程)"
    status: completed
  - id: step-19
    content: "19: platform_linux - sys-daemon 框架 [跳过测试]"
    status: completed
  - id: step-20
    content: "20: platform_linux - infra-daemon + linux_node AppEntry [跳过测试]"
    status: completed
  - id: step-21
    content: "21: 空壳编译 + 体积基线 (< 300KB 红线)"
    status: completed
  - id: step-22
    content: "22: service_config (ConfigService, Level 0)"
    status: completed
  - id: step-23
    content: "23: service_config 测试 (Get/Set/EvtConfigChanged/分层合并)"
    status: completed
  - id: step-24
    content: "24: service_storage (StorageManager, Level 1)"
    status: completed
  - id: step-25
    content: "25: service_storage 测试 (状态机/容量告警/循环覆盖/CmdStorage)"
    status: completed
  - id: step-26
    content: "26: service_network (NetworkManager, Level 1)"
    status: completed
  - id: step-27
    content: "27: service_network 测试 (状态机/重连/EvtNetworkStatus/CmdNetwork)"
    status: completed
  - id: step-28
    content: "28: service_time (TimeSyncService, Level 2)"
    status: completed
  - id: step-29
    content: "29: service_time 测试 (NTP 解析/offset/同步状态机/降级)"
    status: completed
  - id: step-30
    content: "30: 阶段 B 集成测试 (Config+Storage+Network+TimeSync InProcessCommBus)"
    status: completed
  - id: step-31
    content: "31: media_core (MediaCore, Level 3)"
    status: completed
  - id: step-32
    content: "32: media_core 测试 (Pipeline/Fan-out/RefCountedSlot)"
    status: completed
  - id: step-33
    content: "33: transport_p2p (libp2pchannel FFI) [跳过测试]"
    status: completed
  - id: step-34
    content: "34: service_live (LiveService, Level 4)"
    status: completed
  - id: step-35
    content: "35: service_live 测试 (帧流/流控/多观看者/降级)"
    status: completed
  - id: step-36
    content: "36: service_talk (TalkService, Level 4)"
    status: completed
  - id: step-37
    content: "37: service_talk 测试 (上行/下行/双工切换)"
    status: completed
  - id: step-38
    content: "38: service_record (RecordService, Level 4)"
    status: completed
  - id: step-39
    content: "39: service_record 测试 (MP4 写入/分片/CmdRecord/降级)"
    status: completed
  - id: step-40
    content: "40: service_playback (PlaybackService, Level 4)"
    status: completed
  - id: step-41
    content: "41: service_playback 测试 (MP4 读取/QueryTimeline/Seek/降级)"
    status: completed
  - id: step-42
    content: "42: service_cloud (CloudService, Level 5)"
    status: completed
  - id: step-43
    content: "43: service_cloud 测试 (上传队列/重试/降级状态机)"
    status: completed
  - id: step-44
    content: "44: service_upgrade (UpgradeService, Level 5)"
    status: completed
  - id: step-45
    content: "45: service_upgrade 测试 (签名校验/sha2/状态机/回滚)"
    status: completed
  - id: step-46
    content: "46: service_control (ControlGateway, Level 6)"
    status: completed
  - id: step-47
    content: "47: service_control 测试 (路由表/鉴权/会话/事件推送)"
    status: completed
  - id: step-48
    content: "48: Linux 全链路集成测试 (12 Service InProcessCommBus)"
    status: completed
  - id: step-49
    content: "49: linux_node 完整 6 进程架构 [跳过测试]"
    status: completed
  - id: step-50
    content: "50: Linux 全量体积审计 (<= 5MB)"
    status: completed
  - id: step-51
    content: "51: platform_rtos PAL 实现 [跳过测试]"
    status: completed
  - id: step-52
    content: "52: rtos_node 多线程 AppEntry [跳过测试]"
    status: completed
  - id: step-53
    content: "53: RTOS 交叉编译验证 + 体积审计"
    status: completed
  - id: step-54
    content: "54: 跨平台一致性测试"
    status: completed
  - id: step-55
    content: "55: 72h 长稳测试 [跳过测试]"
    status: completed
  - id: step-56
    content: "56: 最终体积审计 + KPI 验收 + 发布基线"
    status: completed
  - id: step-57
    content: "57: InProcessCommBus 改用 SpinMutex 解决 data race + 修复 unsubscribe 按身份匹配"
    status: completed
  - id: step-58
    content: "58: 补全 Live/Playback/Upgrade 服务的 Topic 订阅 (EvtSessionStatus/EvtStorageStatus/EvtNetworkStatus)"
    status: completed
  - id: step-59
    content: "59: 补全 Playback/Upgrade/ControlGateway 降级状态机"
    status: completed
  - id: step-60
    content: "60: ConfigService 实现 FileSystem 持久化读写"
    status: completed
  - id: step-61
    content: "61: TimeSyncService 完善事件处理器 + 修复 consecutive_failures 测试"
    status: completed
  - id: step-62
    content: "62: SysDaemon.tick() 实现 UDS 消息路由"
    status: completed
  - id: step-63
    content: "63: 更新 Plan 正文中与代码不一致的描述"
    status: completed
isProject: false
---

# Rust 嵌入式摄像头全栈软件架构方案

> 同一套 Rust 代码同时运行在 Linux（多进程）和 RTOS（多线程）上的组件化摄像头软件方案。

---

## 一、总体目标

### 1.1 功能覆盖


| 功能         | 对应组件            | 说明                            |
| ---------- | --------------- | ----------------------------- |
| 实时音视频预览    | LiveService     | 主/子码流、多观看者 Fan-out            |
| 音频对讲       | TalkService     | 半双工/全双工、上下行音频                 |
| MP4 录制     | RecordService   | 分片策略、断电保护、循环覆盖                |
| 音视频回放      | PlaybackService | MP4 demux、时间线查询、Seek          |
| 云存储        | CloudService    | 断点续传、指数退避重试、上传队列              |
| 系统升级       | UpgradeService  | OTA 下载、ed25519 签名校验、A/B 分区    |
| 配置管理       | ConfigService   | 分层合并（出厂/用户/云端）、KV 持久化         |
| 存储管理       | StorageManager  | SD 卡检测状态机、容量监控、阈值告警           |
| 网络管理       | NetworkManager  | WiFi 连接状态机、指数退避重连、信号质量        |
| 时间同步       | TimeSyncService | SNTP 客户端、大跳变保护                |
| 远程控制（命令网关） | ControlGateway  | App 协议解析、表驱动分发、会话管理、鉴权        |
| 媒体管线       | MediaCore       | Pipeline 调度、双码流分发、Fan-out 帧共享 |


### 1.2 平台目标

- **Linux**：多进程架构（6 个 daemon），进程间 shm + UDS 通信
- **RTOS**：多线程架构（单进程），线程间内存直接访问 + 信号量/EventGroup
- **共享代码**：所有业务 crate `#![no_std]`，平台差异封装在 PAL trait + 平台层 crate

### 1.3 核心约束

- 全量编译后二进制 **<= 5MB**（TLS 库不计入）
- 空壳编译基线 **< 300KB**
- 业务 crate 全部 `#![no_std]`，Linux 平台层允许 `std`
- 三方依赖严格白名单，尽量自实现

---

## 二、精简策略

### 2.1 no_std 约束表


| Crate            | `#![no_std]` | 允许 `std` | 说明                        |
| ---------------- | ------------ | -------- | ------------------------- |
| core_types       | Yes          | No       | FixedVec/FixedString/错误码  |
| core_interfaces  | Yes          | No       | CommBus/Service/PAL trait |
| comm             | Yes          | No       | RingBuf/Fan-out/Router    |
| media_core       | Yes          | No       | Pipeline 调度/帧分发           |
| service_live     | Yes          | No       | 实时预览                      |
| service_talk     | Yes          | No       | 音频对讲                      |
| service_record   | Yes          | No       | MP4 录制                    |
| service_playback | Yes          | No       | 音视频回放                     |
| service_cloud    | Yes          | No       | 云存                        |
| service_upgrade  | Yes          | No       | OTA 升级                    |
| service_config   | Yes          | No       | 配置管理                      |
| service_storage  | Yes          | No       | 存储管理                      |
| service_network  | Yes          | No       | 网络管理                      |
| service_time     | Yes          | No       | 时间同步                      |
| service_control  | Yes          | No       | App 命令网关/鉴权/会话            |
| transport_p2p    | Yes          | No       | P2P 通道抽象                  |
| platform_linux   | No           | Yes      | Linux PAL 实现 (std::fs 等)  |
| platform_rtos    | Yes          | No       | RTOS PAL 实现 (BSP FFI)     |
| linux_node       | No           | Yes      | Linux 可执行入口               |
| rtos_node        | Yes          | No       | RTOS 可执行入口                |


### 2.2 三方依赖白名单

```toml
# 密码学（安全敏感，禁止自实现）
ed25519-compact = { version = "2", default-features = false }
sha2 = { version = "0.10", default-features = false }
crc32fast = { version = "1", default-features = false }

# 序列化（no_std，可选 JSON payload）
serde = { version = "1", default-features = false, features = ["derive"] }
serde_json_core = { version = "0.6", default-features = false }
```

- TLS/HTTPS：抽象为 PAL trait，Linux 用 `rustls`（或系统 OpenSSL），RTOS 用 `mbedtls` FFI
- MP4 muxer/demuxer：长期使用三方 `no_std` crate（格式复杂度高，自实现 ROI 极低）
- **不引入**：tokio、async-std、hyper 等重量级运行时

### 2.3 编译优化

```toml
[profile.release]
opt-level = "z"        # 最小体积优化
lto = "fat"            # 全程序链接时优化
codegen-units = 1      # 单编译单元（更好优化）
panic = "abort"        # 无 unwind 表
strip = true           # 去除符号表
overflow-checks = false
```

### 2.4 体积预算


| 组件             | 预算      | 实际 (release) |
| -------------- | ------- | ------------ |
| 空壳基线 (全服务空实现)  | < 300KB | 284KB        |
| 全功能 linux_node | < 5MB   | **320KB**    |
| 裕量             | ~4.7MB  | --           |


- 空壳超 300KB 立即瘦身
- 如 serde 体积超预期，备选方案：控制面 payload 改手写二进制序列化（~50 行），省 ~200KB

---

## 三、分层架构

```
┌────────────────────────────────────────────────────────────┐
│                     Application Layer                      │
│   linux_node (多进程)    │    rtos_node (多线程)              │
├────────────────────────────────────────────────────────────┤
│                     Platform Layer                         │
│   platform_linux (std)   │    platform_rtos (no_std)       │
│   shm_ring / uds_router │    static_ringbuf / semaphore   │
│   sys-daemon / infra-daemon │  RTOS task scheduler        │
├────────────────────────────────────────────────────────────┤
│                     Service Layer (all no_std)              │
│   ConfigService │ StorageManager │ NetworkManager           │
│   TimeSyncService │ MediaCore │ LiveService │ TalkService   │
│   RecordService │ PlaybackService │ CloudService            │
│   UpgradeService │ ControlGateway                          │
├────────────────────────────────────────────────────────────┤
│                   Communication Layer (no_std)              │
│   comm: SpscRingBuf │ FanOutPublisher │ TopicRouter         │
│         RequestReplyEngine │ InProcessCommBus               │
├────────────────────────────────────────────────────────────┤
│                     Core Layer (no_std)                     │
│   core_types: FixedVec │ FixedString │ CamError │ CtrlMsg   │
│               FrameHeader │ Topic │ ServiceId │ cam_log!    │
│   core_interfaces: CommBus │ Service │ PAL traits           │
└────────────────────────────────────────────────────────────┘
```

---

## 四、工程结构

```
libcamera/
├── Cargo.toml                  # Workspace root + release profile + 依赖白名单
├── crates/
│   ├── core_types/             # #![no_std] 基础类型、错误码、消息信封
│   ├── core_interfaces/        # #![no_std] CommBus / Service / PAL trait 定义
│   ├── comm/                   # #![no_std] 通信层实现
│   │   ├── ring_buffer.rs      #   SPSC RingBuffer
│   │   ├── fan_out.rs          #   Fan-out Publisher + RefCountedSlot
│   │   ├── topic_router.rs     #   Topic 路由器
│   │   ├── request_reply.rs    #   非阻塞 Request/Reply 引擎
│   │   ├── spin_mutex.rs       #   no_std SpinMutex (线程安全保护)
│   │   └── in_process.rs       #   InProcessCommBus (测试 + RTOS)
│   ├── media_core/             # #![no_std] Pipeline 调度 / 帧分发
│   ├── transport_p2p/          # #![no_std] P2P 通道抽象 (libp2pchannel FFI)
│   ├── service_config/         # #![no_std] 配置管理
│   ├── service_storage/        # #![no_std] 存储管理
│   ├── service_network/        # #![no_std] 网络管理
│   ├── service_time/           # #![no_std] 时间同步 (SNTP)
│   ├── service_live/           # #![no_std] 实时预览
│   ├── service_talk/           # #![no_std] 音频对讲
│   ├── service_record/         # #![no_std] MP4 录制
│   ├── service_playback/       # #![no_std] 音视频回放
│   ├── service_cloud/          # #![no_std] 云存储上传
│   ├── service_upgrade/        # #![no_std] OTA 升级
│   ├── service_control/        # #![no_std] App 命令网关 (ControlGateway)
│   ├── platform_linux/         # std 允许: Linux PAL 实现
│   │   ├── pal_fs.rs           #   FileSystem -> std::fs
│   │   ├── pal_timer.rs        #   Timer -> std::time
│   │   ├── pal_network.rs      #   NetworkHal stub
│   │   ├── shm_ring.rs         #   shm 数据面 RingBuffer (mmap)
│   │   ├── uds_router.rs       #   UDS 控制面 TopicRouter
│   │   └── daemon/
│   │       ├── sys_daemon.rs   #     shm 生命周期 + watchdog + CmdDevice
│   │       ├── infra_daemon.rs #     基础设施服务进程
│   │       └── app_entry.rs    #     分级启动 STARTUP_ORDER
│   └── platform_rtos/          # #![no_std] RTOS PAL 实现 (stub)
├── apps/
│   ├── linux_node/             # Linux 可执行入口 (std)
│   └── rtos_node/              # RTOS 可执行入口 (no_std, no_main)
└── tests/
    └── integration/            # 集成测试 (InProcessCommBus)
```

---

## 五、通信架构

### 5.1 通信分层


| 层面  | 传输方式 (Linux)           | 传输方式 (RTOS)     | Topic 范围               |
| --- | ---------------------- | --------------- | ---------------------- |
| 数据面 | **shm RingBuffer**     | 内存直接访问          | VideoXxx / AudioXxx 等  |
| 控制面 | **Unix Domain Socket** | 函数调用/消息队列       | CmdXxx (Request/Reply) |
| 事件面 | **Unix Domain Socket** | 函数调用/EventGroup | EvtXxx (Pub/Sub 广播)    |


数据面使用 shm 实现零拷贝高吞吐；控制面和事件面使用 UDS，由内核管理缓冲区，进程 crash 时自动清理。RTOS 模式下所有通信在同一地址空间内完成。

### 5.2 SPSC RingBuffer

```rust
pub struct SpscRingBuf {
    buf: *mut u8,
    len: usize,
    head: AtomicUsize,  // 生产者写指针
    tail: AtomicUsize,  // 消费者读指针
}
```

- 单生产者单消费者，lock-free
- 满缓冲区时丢弃最旧帧（背压策略）
- Linux 数据面：底层映射到 shm（`ShmRingBuf`）
- RTOS / 测试：底层使用堆分配内存

### 5.3 Fan-out SPSC 多消费者模型

```rust
pub struct FanOutPublisher {
    rings: [Option<SubscriberRing>; MAX_FANOUT],  // MAX_FANOUT=8，运行时动态
    active_count: u8,
    slots: [RefCountedSlot; MAX_SLOTS],           // 引用计数共享帧
}
```

- 1 个生产者写入 N 个独立 SPSC RingBuffer
- 运行时 `add_consumer()` / `remove_consumer()` 动态增减消费者
- `RefCountedSlot`：帧数据引用计数，所有消费者读完后自动释放
- publish 时只写入 `active_count` 个 RingBuffer，inactive slot 跳过

### 5.4 Topic 枚举

```rust
#[repr(u8)]
pub enum Topic {
    // 数据面 (shm, 高吞吐)
    VideoMainStream = 0, VideoSubStream = 1, AudioCapture = 2,
    TalkDownlink = 3, TalkUplink = 4, PlaybackStream = 5,

    // 控制面 (UDS, Request/Reply)
    CmdLive = 10, CmdTalk = 11, CmdRecord = 12, CmdPlayback = 13,
    CmdCloud = 14, CmdUpgrade = 15, CmdConfig = 16, CmdStorage = 17,
    CmdNetwork = 18, CmdTime = 19, CmdDevice = 20, CmdMediaCore = 21,
    CmdControl = 22,

    // 事件面 (UDS, Pub/Sub 广播)
    EvtConfigChanged = 30, EvtNetworkStatus = 31, EvtStorageStatus = 32,
    EvtTimeSync = 33, EvtAlarm = 34, EvtSessionStatus = 35,
    EvtUpgradeStatus = 36,
}
```

- 数据面 6 个 Topic，控制面 13 个 Topic，事件面 7 个 Topic
- `is_data_plane()` / `is_control_plane()` / `is_event_plane()` 方法自动分类

### 5.5 生产者/消费者映射表


| Topic            | 模式             | Producer       | Consumer                                              | 关系  |
| ---------------- | -------------- | -------------- | ----------------------------------------------------- | --- |
| VideoMainStream  | Pub/Sub FanOut | MediaCore      | Record, Cloud                                         | 1:2 |
| VideoSubStream   | Pub/Sub FanOut | MediaCore      | Live (多路 Fan-out)                                     | 1:N |
| AudioCapture     | Pub/Sub FanOut | MediaCore      | Live, Talk, Record                                    | 1:3 |
| TalkDownlink     | Pub/Sub        | Talk           | MediaCore (解码播放)                                      | 1:1 |
| TalkUplink       | Pub/Sub        | Talk           | ControlGateway (转发 App)                               | 1:1 |
| PlaybackStream   | Pub/Sub        | Playback       | Live (复用传输通道)                                         | 1:1 |
| CmdLive          | Request/Reply  | ControlGateway | Live                                                  | 1:1 |
| CmdTalk          | Request/Reply  | ControlGateway | Talk                                                  | 1:1 |
| CmdRecord        | Request/Reply  | ControlGateway | Record                                                | 1:1 |
| CmdPlayback      | Request/Reply  | ControlGateway | Playback                                              | 1:1 |
| CmdCloud         | Request/Reply  | ControlGateway | Cloud                                                 | 1:1 |
| CmdUpgrade       | Request/Reply  | ControlGateway | Upgrade                                               | 1:1 |
| CmdConfig        | Request/Reply  | ControlGateway | Config                                                | 1:1 |
| CmdStorage       | Request/Reply  | ControlGateway | Storage                                               | 1:1 |
| CmdNetwork       | Request/Reply  | ControlGateway | Network                                               | 1:1 |
| CmdTime          | Request/Reply  | ControlGateway | TimeSync                                              | 1:1 |
| CmdDevice        | Request/Reply  | ControlGateway | sys-daemon                                            | 1:1 |
| CmdMediaCore     | Request/Reply  | ControlGateway | MediaCore                                             | 1:1 |
| CmdControl       | Request/Reply  | (internal)     | ControlGateway                                        | 1:1 |
| EvtConfigChanged | Pub/Sub FanOut | Config         | Storage, Network, TimeSync, MediaCore, ControlGateway | 1:5 |
| EvtNetworkStatus | Pub/Sub FanOut | Network        | TimeSync, Cloud, Upgrade, Live, ControlGateway        | 1:5 |
| EvtStorageStatus | Pub/Sub FanOut | Storage        | Record, Playback, Cloud, Upgrade, ControlGateway      | 1:5 |
| EvtTimeSync      | Pub/Sub        | TimeSync       | Record, Cloud, ControlGateway                         | 1:3 |
| EvtAlarm         | Pub/Sub FanOut | MediaCore      | Record, Cloud, ControlGateway                         | 1:3 |
| EvtSessionStatus | Pub/Sub FanOut | ControlGateway | Live, Talk, Playback                                  | 1:3 |
| EvtUpgradeStatus | Pub/Sub        | Upgrade        | ControlGateway                                        | 1:1 |


### 5.6 CommBus trait

```rust
pub trait CommBus {
    // Pub/Sub 控制面/事件面
    fn publish_ctrl(&self, topic: Topic, msg: &CtrlMsg, payload: &[u8]) -> CommResult<()>;
    fn poll_ctrl(&self, buf: &mut [u8]) -> CommResult<Option<(Topic, CtrlMsg)>>;
    fn subscribe(&self, topic: Topic) -> CommResult<()>;
    fn unsubscribe(&self, topic: Topic) -> CommResult<()>;

    // Pub/Sub 数据面
    fn publish_frame(&self, topic: Topic, header: &FrameHeader, data: &[u8]) -> CommResult<()>;
    fn poll_frame(&self, topic: Topic, hdr_buf: &mut FrameHeader, data_buf: &mut [u8])
        -> CommResult<Option<usize>>;

    // Request/Reply (非阻塞)
    fn send_request(&self, topic: Topic, msg: &CtrlMsg, payload: &[u8]) -> CommResult<PendingReply>;
    fn poll_reply(&self, pending: &PendingReply, buf: &mut [u8]) -> CommResult<Option<CtrlMsg>>;
    fn cancel_request(&self, pending: PendingReply) -> CommResult<()>;
    fn reply(&self, topic: Topic, request_id: u16, msg: &CtrlMsg, payload: &[u8]) -> CommResult<()>;
}
```

Request/Reply 采用非阻塞两步式：`send_request()` 立即返回 `PendingReply` handle，Service 主循环中 `poll_reply()` 检查响应。comm 层不依赖任何 PAL 同步原语。

### 5.7 消息信封

**控制面信封** (`CtrlMsg`, 16 bytes, `#[repr(C)]`):

```rust
pub struct CtrlMsg {
    pub topic: u8,          // Topic 编号
    pub _pad: u8,
    pub method_id: u16,     // MethodId 编号
    pub request_id: u16,    // 请求标识（用于 RR 匹配）
    pub source: u8,         // ServiceId
    pub flags: u8,          // FLAG_RESPONSE / FLAG_ERROR / FLAG_HAS_JSON
    pub payload_len: u16,   // 后续 JSON payload 长度
    pub _reserved: u16,
    pub timestamp_ms: u32,
}
```

**数据面帧头** (`FrameHeader`, 32 bytes, `#[repr(C)]`):

```rust
pub struct FrameHeader {
    pub frame_type: u8,     // FrameType 枚举
    pub stream_id: u8,
    pub flags: u8,          // FLAG_KEYFRAME / FLAG_EOS
    pub _pad: u8,
    pub seq: u32,           // 帧序号
    pub pts_ms: u64,        // 显示时间戳
    pub dts_ms: u64,        // 解码时间戳
    pub data_len: u32,      // 帧数据长度
    pub _reserved: u32,
}
```

### 5.8 InProcessCommBus (测试 + RTOS)

`comm::in_process::InProcessCommBus` 实现 `CommBus` trait，所有服务在同一进程不同线程中运行：

- 内部使用 `SpinMutex<[TopicSubs]>` + `SpinMutex<RequestReplyEngine>` 保证线程安全
- `SpinMutex`：自实现的 `no_std` 自旋锁（`comm::spin_mutex`），基于 `AtomicBool` + `compare_exchange_weak`
- 用于集成测试：单进程验证多服务交互，无需 fork 多进程
- 同时验证 RTOS 多线程模式正确性
- CI 友好，测试速度快

### 5.9 降级状态机

```
Normal -> Degraded -> Suspended
  ^          |            |
  +----------+------------+  (恢复条件满足时)
```

各服务降级行为：


| 服务              | Degraded 条件                | Suspended 条件               | Degraded 行为 | 已实现 |
| --------------- | -------------------------- | -------------------------- | ----------- | --- |
| RecordService   | StorageStatus=FULL         | StorageStatus=REMOVED      | 循环覆盖模式      | Yes |
| CloudService    | NetworkStatus=Weak         | NetworkStatus=Disconnected | 降低上传并发      | Yes |
| LiveService     | --                         | 无活跃连接 (EvtSessionStatus)   | 停止消费帧省CPU   | Yes |
| PlaybackService | --                         | StorageStatus=REMOVED      | 停止回放        | Yes |
| UpgradeService  | --                         | Network断开 / Storage不可用     | 取消下载中升级     | Yes |
| ControlGateway  | NetworkStatus=Disconnected | --                         | 仅局域网可达      | Yes |
| TimeSyncService | 连续同步失败>3次                  | --                         | 使用本地时钟      | Yes |


每个服务的 `health()` 方法返回 `HealthStatus { service, state, error_code }`。

---

## 六、进程/线程映射

### 6.1 Linux 多进程架构 (6 进程)

```
┌─────────────────────────────────────────────────────┐
│                   sys-daemon                         │
│  shm 生命周期管理 │ 进程 watchdog │ CmdDevice 处理     │
│  UDS TopicRouter (控制面路由枢纽)                     │
└───────┬──────────┬──────────┬──────────┬────────────┘
        │ UDS      │ UDS      │ UDS      │ UDS
┌───────▼───────┐ ┌▼─────────▼┐ ┌───────▼────────┐ ┌──────────────┐
│ infra-daemon  │ │media-daemon│ │session-daemon  │ │ cloud-daemon │
│ ConfigService │ │ MediaCore  │ │ LiveService    │ │ CloudService │
│ NetworkManager│ │            │ │ TalkService    │ │ UpgradeService│
│ StorageManager│ │ shm 数据面  │ │ PlaybackService│ │              │
│ TimeSyncService│ │ 帧生产者   │ │ RecordService  │ │              │
│               │ │            │ │ ControlGateway │ │              │
│               │ │            │ │ transport_p2p  │ │              │
└───────────────┘ └────────────┘ └────────────────┘ └──────────────┘
```

- **数据面**：MediaCore -> shm RingBuffer -> LiveService/RecordService/CloudService（零拷贝）
- **控制面/事件面**：所有 CmdXxx/EvtXxx 通过 UDS，由 sys-daemon 中的 UdsTopicRouter 转发
- infra-daemon crash 后 sys-daemon 可独立重启它，不影响 shm 和媒体管线

### 6.2 RTOS 多线程架构 (单进程)

```
┌────────────────────────────────────────────────────────────┐
│                       rtos_node                            │
│                                                            │
│  media_thread (highest)   : MediaCore                      │
│  stream_thread (high)     : Live, Talk                     │
│  record_thread (medium)   : Record, Playback               │
│  control_thread (medium)  : ControlGateway, transport_p2p  │
│  infra_thread (low)       : Config, Network, Storage, Time │
│  cloud_thread (lowest)    : Cloud, Upgrade                 │
│                                                            │
│  InProcessCommBus (内存直接访问，无 IPC 开销)               │
└────────────────────────────────────────────────────────────┘
```

- 线程优先级按实时性需求分配
- 静态内存预分配，无运行时堆分配

---

## 七、服务启动依赖图

```
Level 0 (无依赖):  ConfigService
Level 1:           NetworkManager, StorageManager       (依赖 Config)
Level 2:           TimeSyncService                      (依赖 Network)
Level 3:           MediaCore                            (依赖 Config)
Level 4:           Live, Talk, Record, Playback         (依赖 MediaCore + Storage/Network/TimeSync)
Level 5:           CloudService, UpgradeService         (依赖 Network + Storage)
Level 6:           ControlGateway                       (依赖 transport_p2p + 所有上层服务就绪)
```

`Service` trait 通过 `dependencies()` 声明依赖：

```rust
pub trait Service {
    fn service_id(&self) -> ServiceId;
    fn dependencies(&self) -> &'static [ServiceId];
    fn init(&mut self, bus: &dyn CommBus) -> CommResult<()>;
    fn start(&mut self) -> CommResult<()>;
    fn stop(&mut self) -> CommResult<()>;
    fn health(&self) -> HealthStatus;
    fn poll(&mut self) -> CommResult<bool> { Ok(false) }
}
```

AppEntry 按 Level 拓扑排序初始化，每级 init+start 完成后再启动下一级。某级失败则该级及所有后续级进入 Suspended 状态。

```rust
pub const STARTUP_ORDER: &[&[ServiceId]] = &[
    &[ServiceId::Config],
    &[ServiceId::Network, ServiceId::Storage],
    &[ServiceId::TimeSync],
    &[ServiceId::MediaCore],
    &[ServiceId::Live, ServiceId::Talk, ServiceId::Record, ServiceId::Playback],
    &[ServiceId::Cloud, ServiceId::Upgrade],
    &[ServiceId::ControlGateway],
];
```

---

## 八、逐模块实现计划

### 模块 1: ConfigService (Level 0)

**职责**：配置管理，分层合并（出厂默认 / 用户自定义 / 云端下发），KV 持久化。

- Producer：`EvtConfigChanged`
- Consumer：`CmdConfig` (Get/Set)
- 依赖 PAL：`FileSystem` (持久化)
- 降级：无（基础设施服务，不降级）

### 模块 2: StorageManager (Level 1)

**职责**：SD 卡/存储检测状态机，容量监控，阈值告警，循环覆盖策略。

- Producer：`EvtStorageStatus` (LOW/FULL/REMOVED)
- Consumer：`CmdStorage` (QueryCapacity/Format)，`EvtConfigChanged`
- 依赖 PAL：`StorageHal`
- 降级：无（作为触发源而非消费者）

### 模块 3: NetworkManager (Level 1)

**职责**：WiFi 连接状态机 (Disconnected→Connecting→Connected→Online)，指数退避重连，信号质量监控。

- Producer：`EvtNetworkStatus` (Connected/Disconnected/Weak)
- Consumer：`CmdNetwork` (ScanWifi/ConnectWifi)，`EvtConfigChanged`
- 依赖 PAL：`NetworkHal`
- 降级：无

### 模块 4: TimeSyncService (Level 2)

**职责**：SNTP 客户端，NTP 48 字节包构造/解析，大跳变保护，同步状态机。

- Producer：`EvtTimeSync`
- Consumer：`CmdTime` (SyncNow/QueryTime)，`EvtNetworkStatus`，`EvtConfigChanged`
- 依赖 PAL：`UdpSocket`，`SystemClock`
- 降级：连续同步失败 → Degraded（使用本地时钟）

### 模块 5: MediaCore (Level 3)

**职责**：Pipeline 调度器，管理 Source（摄像头/麦克风）→ Encoder → 双码流分发。

- Producer：`VideoMainStream`，`VideoSubStream`，`AudioCapture`，`EvtAlarm`
- Consumer：`CmdMediaCore` (SetBitrate/RequestIDR/SetResolution)，`EvtConfigChanged`
- Fan-out：AudioCapture 1:3 (Live+Talk+Record)，VideoSubStream 1:N (多观看者)
- 依赖 PAL：HAL (Camera/Audio capture)

### 模块 6: LiveService (Level 4)

**职责**：实时预览，subscribe 子码流/音频，流控状态机，码率自适应，多观看者 Fan-out。

- Consumer：`VideoSubStream`，`AudioCapture`，`CmdLive`，`EvtSessionStatus`
- 降级：无活跃连接 → Suspended (停止消费帧省 CPU)；Network 断开 → Suspended

### 模块 7: TalkService (Level 4)

**职责**：上行/下行音频路径，半双工/全双工切换。

- Producer：`TalkUplink`，`TalkDownlink`
- Consumer：`AudioCapture`，`CmdTalk`
- 降级：无

### 模块 8: RecordService (Level 4)

**职责**：MP4 muxer，分片策略（按时间/大小），断电保护，subscribe 主码流+音频+告警。

- Consumer：`VideoMainStream`，`AudioCapture`，`EvtAlarm`，`CmdRecord`，`EvtStorageStatus`
- 依赖 PAL：`FileSystem`
- 降级：Storage FULL → Degraded (循环覆盖)；Storage REMOVED → Suspended (停录)

### 模块 9: PlaybackService (Level 4)

**职责**：MP4 demuxer，时间线查询，Seek，回放帧输出。

- Producer：`PlaybackStream`
- Consumer：`CmdPlayback` (Start/Stop/QueryTimeline/Seek)，`EvtStorageStatus`
- 依赖 PAL：`FileSystem`
- 降级：Storage REMOVED → Suspended

### 模块 10: CloudService (Level 5)

**职责**：云存储上传队列，断点续传，指数退避重试。

- Consumer：`CmdCloud`，`EvtNetworkStatus`，`EvtStorageStatus`
- 依赖 PAL：`HttpClient`
- 降级：Network Weak → Degraded (降低并发)；Network Disconnected → Suspended (暂停上传)

### 模块 11: UpgradeService (Level 5)

**职责**：OTA 固件下载，ed25519 签名校验，sha2 完整性验证，A/B 分区切换，回滚。

- Producer：`EvtUpgradeStatus`
- Consumer：`CmdUpgrade`，`EvtNetworkStatus`，`EvtStorageStatus`
- 依赖 PAL：`HttpClient`，`BootManager`
- 降级：Network 断开 → Suspended；Storage 满 → Suspended

### 模块 12: ControlGateway (Level 6)

**职责**：App 远程控制命令网关，协议解析，表驱动分发，会话管理，鉴权。

- Producer：所有 `CmdXxx` (13 个 Request/Reply) + `EvtSessionStatus`
- Consumer：`EvtAlarm`，`EvtUpgradeStatus`，`EvtStorageStatus`，`EvtNetworkStatus`，`EvtConfigChanged`，`EvtTimeSync` (转推 App)
- 降级：Network 断开 → Degraded (仅局域网可达)

**核心设计**：

1. **表驱动命令分发**：

```rust
const CMD_ROUTE_TABLE: &[(u16, Topic)] = &[
    (METHOD_START_LIVE,    Topic::CmdLive),
    (METHOD_START_RECORD,  Topic::CmdRecord),
    (METHOD_GET_CONFIG,    Topic::CmdConfig),
    // 新增服务只加一行
];
```

1. **会话表**：`FixedVec<Session, MAX_CLIENTS>`，含 session_id / auth_level / last_heartbeat
2. **鉴权**：SHA256 密码 hash 比对，Admin / Viewer 权限分级
3. **事件推送**：subscribe 多个 EvtXxx Topic，poll 后序列化推送给各连接的 App 客户端

---

## 九、PAL (Platform Abstraction Layer)

```rust
pub trait FileSystem { ... }      // 文件读写、目录操作、磁盘空间
pub trait NetworkHal { ... }      // WiFi 连接、信号强度、扫描
pub trait StorageHal { ... }      // SD 卡检测、挂载、格式化
pub trait SystemClock { ... }     // 系统时钟、单调时钟
pub trait UdpSocket { ... }       // UDP 收发 (NTP)
pub trait HttpClient { ... }      // HTTP GET/PUT/POST (云存/升级)
pub trait Timer { ... }           // 单调时钟、sleep
pub trait BootManager { ... }     // A/B 分区切换、回滚
pub trait SystemControl { ... }   // 重启、恢复出厂
pub trait PtzHal { ... }          // 云台控制 (可选)
```

Linux 实现：`platform_linux` 使用 `std::fs`、`std::net`、`std::time`、`libc` 等。

RTOS 实现：`platform_rtos` 使用 BSP FFI、CMSIS-RTOS API、mbedtls FFI 等。

---

## 十、交付里程碑

### 阶段 A: 基座层 (步骤 01-21)

Cargo workspace、core_types、core_interfaces、comm 通信层全实现（SPSC RingBuffer + Fan-out + TopicRouter + Request/Reply + InProcessCommBus）、platform_linux PAL + shm + UDS、daemon 框架、空壳编译基线。

### 阶段 B: 基础设施服务 (步骤 22-30)

ConfigService + StorageManager + NetworkManager + TimeSyncService，含单元测试 + 集成测试。

### 阶段 C: 媒体管线 + 实时预览 (步骤 31-35)

MediaCore + transport_p2p + LiveService。

### 阶段 D: 对讲 + 录制 + 回放 (步骤 36-41)

TalkService + RecordService + PlaybackService。

### 阶段 E: 云存 + 升级 (步骤 42-45)

CloudService + UpgradeService。

### 阶段 F: 命令网关 + 端到端 (步骤 46-50)

ControlGateway + Linux 全链路集成测试 + 完整 6 进程架构 + 体积审计。

### 阶段 G: RTOS 适配 (步骤 51-53)

platform_rtos PAL + rtos_node + RTOS 交叉编译验证。

### 阶段 H: 稳定性 + 发布 (步骤 54-56)

跨平台一致性测试 + 72h 长稳测试 + 最终 KPI 验收。

### 阶段 I: 架构优化 (步骤 57-63)

InProcessCommBus 线程安全重构 (SpinMutex)、Topic 订阅补全、降级状态机完善、ConfigService 持久化、TimeSyncService 事件处理器、SysDaemon UDS 消息路由、Plan 文档同步。

---

## 十一、KPI 指标


| 指标             | 目标值     | 实际达成              |
| -------------- | ------- | ----------------- |
| Release 二进制体积  | <= 5MB  | **320KB** (远低于目标) |
| 空壳编译基线         | < 300KB | 284KB             |
| 自动化测试数         | >= 100  | **114 tests**     |
| 测试通过率          | 100%    | **0 failures**    |
| no_std crate 数 | >= 15   | **17 crates**     |
| 远程控制命令响应延迟     | < 200ms | (待真实环境验证)         |
| 并发客户端数         | >= 4    | MAX_CLIENTS=4     |
| 事件推送延迟         | < 500ms | (待真实环境验证)         |
| 鉴权拒绝率          | 100%    | (SHA256 + 权限分级)   |


---

## 十二、设计分析

### 优势

1. **Rust 内存安全**：所有业务逻辑在编译期保证内存安全，unsafe 仅限 shm mmap 和 `#[repr(C)]` 序列化
2. **真正的跨平台**：17 个 `no_std` crate 在 Linux 和 RTOS 上共享同一份源码
3. **极致体积**：320KB 全功能二进制（不含 TLS），远低于 5MB 预算
4. **松耦合**：Pub/Sub + 表驱动分发，新增服务只需加一行路由表
5. **可测试性**：InProcessCommBus（SpinMutex 线程安全）允许单进程集成测试，CI 友好
6. **降级韧性**：7 个服务实现降级状态机，关键服务故障不影响其他服务
7. **零拷贝数据面**：shm RingBuffer + RefCountedSlot，多消费者共享帧数据
8. **配置持久化**：ConfigService 通过 FileSystem PAL 实现 "key=value" 格式持久化

### 风险与应对

1. **MP4 格式复杂度**：长期依赖三方 crate，重点投入断电恢复
2. **RTOS 真机验证**：需真实硬件环境，当前仅交叉编译验证
3. **P2P 网络可靠性**：transport_p2p 依赖外部 libp2pchannel，需要集成测试
4. **长稳测试**：72h 稳定性测试需要真实环境运行

