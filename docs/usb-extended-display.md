# RotaScope USB 扩展显示方案

RotaScope 的目标是把 Android 手机变成电脑的第二块显示器。这个能力由三部分组成：

1. Android App：全屏显示电脑发来的画面。
2. PC 服务端：捕获某个 Windows 显示器画面，编码成视频流，通过网络推给手机。
3. 虚拟显示驱动：让 Windows 真的出现一块可扩展桌面的显示器。

当前项目提供两个版本的服务端：

- **v1 (rotascope-server)**：MVP 版本，WebSocket + JPEG 推流，15 FPS
- **v2 (rotascope_server2)**：开发中，QUIC + H.264 编码，60 FPS

## 项目架构

```
┌──────────────── WINDOWS ────────────────┐
│  IDD Virtual Display Driver (C++)       │
│            ↓                            │
│   DWM Render Target                     │
│            ↓                            │
│   DXGI / scrap Capture                  │
│            ↓                            │
│   JPEG 编码  /  H.264 编码              │
│            ↓                            │
│   USB Transport (ADB reverse)           │
│            ↓                            │
│   Input Controller (SendInput)          │
└───────────────┬─────────────────────────┘
                │ USB 3.0
┌───────────────▼─────────────────────────┐
│              ANDROID                    │
│  Image.memory / MediaCodec H264         │
│            ↓                            │
│  SurfaceView + GestureDetector          │
│            ↓                            │
│  Touch → WebSocket / QUIC 回传          │
└─────────────────────────────────────────┘
```

## 运行 MVP (v1)

准备：

- Android 手机开启开发者选项和 USB 调试。
- 电脑安装 Android platform-tools，并确保 `adb.exe` 在 `PATH` 中。
- 电脑安装 Rust 工具链。
- Flutter 项目依赖已拉取完成。

启动电脑端：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start_usb_display.ps1
```

指定捕获显示器：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start_usb_display.ps1 -DisplayIndex 1
```

启动手机端：

```powershell
cd .\rotascope_app
flutter run -d <android-device-id>
```

手机端默认连接地址是：

```text
127.0.0.1:8083/ws
```

这不是手机自己的服务，而是 `adb reverse` 映射到电脑上的 RotaScope 服务。

## 运行 v2 (QUIC + H.264)

启动电脑端：

```bash
cd rotascope_server2
cargo run --release -- --listen-addr 0.0.0.0:1234
```

v2 使用 QUIC 协议，需要更新 Android 端以支持 H.264 解码和 QUIC 传输（开发中）。

## 变成真正扩展屏

要让 Windows 把手机当作扩展显示器，需要完成电脑端驱动链路：

1. 完善 `VirtualDisplayDriver/` 为可安装的 Windows Indirect Display Driver。
2. 编译生成 `.sys` 驱动和对应 `.inf`。
3. 使用测试签名或正式签名安装驱动。
4. 在 Windows 显示设置中启用新出现的虚拟显示器，并选择"扩展这些显示器"。
5. 运行 RotaScope 服务端，用 `-DisplayIndex` 或环境变量 `ROTASCOPE_DISPLAY_INDEX` 捕获虚拟显示器。
6. 手机端通过 USB 默认地址接收画面。

`rotascope-server` 当前提供的是用户态捕获和 WebSocket 推流，不负责创建 Windows 显示设备。驱动这部分必须在 PC 上完成，Android App 无法替代。

## 端口和协议

### v1 (WebSocket + JPEG)

- PC 服务端监听：`0.0.0.0:8083/ws`
- USB 转发：`adb reverse tcp:8083 tcp:8083`
- 手机连接：`ws://127.0.0.1:8083/ws`
- 视频帧格式：二进制 JPEG WebSocket 消息
- 控制消息：JSON 文本消息

### v2 (QUIC + H.264)

- PC 服务端监听：`0.0.0.0:1234`
- USB 转发：`adb reverse tcp:1234 tcp:1234`
- 手机连接：QUIC 连接到 `127.0.0.1:1234`
- 视频帧格式：H.264 编码的视频流
- 控制消息：QUIC 双向流

## 组件状态

| 组件 | 状态 | 说明 |
|------|------|------|
| rotascope_app | ✅ 完成 | Flutter 显示端，JPEG 解码，触摸回传，陀螺仪切屏 |
| rotascope-server | ✅ MVP | 屏幕捕获 → WebSocket → JPEG 推流，15 FPS |
| rotascope_server2 | 🚧 开发中 | QUIC 传输 + H.264 编码 + SendInput 注入 |
| rotascope-core | ✅ 完成 | 共享协议库 |
| VirtualDisplayDriver | 🚧 开发中 | Windows IDD 驱动框架 |
