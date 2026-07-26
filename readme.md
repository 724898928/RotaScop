# RotaScope

RotaScope 把 Android 手机作为电脑的 USB 扩展显示器使用。

## 项目结构

```
rotascope_app/         Flutter Android 显示端
rotascope-server/      Rust PC 服务端 MVP（WebSocket + JPEG）
rotascope_server2/     Rust PC 服务端 v2（QUIC + H.264，开发中）
rotascope-core/        共享协议库
VirtualDisplayDriver/  Windows 虚拟显示驱动（IDD 雏形）
scripts/               启动脚本
docs/                  文档
```

## 快速启动（MVP）

### 1. 电脑端

```powershell
# Windows
powershell -ExecutionPolicy Bypass -File .\scripts\start_usb_display.ps1
```

```bash
# macOS / Linux
chmod +x scripts/start_usb_display.sh
./scripts/start_usb_display.sh
```

### 2. Android 端

```bash
cd rotascope_app
flutter run -d <android-device-id>
```

手机端默认连接 `ws://127.0.0.1:8083/ws`（通过 `adb reverse` 映射到电脑）。

### 3. 功能

- 全屏显示电脑画面（JPEG 流，15 FPS）
- 触摸回传：手机触摸事件发送到电脑，支持鼠标模拟（v2）
- 陀螺仪切屏：旋转手机切换显示器
- 自动重连：指数退避重连
- HUD 显示：FPS、显示器编号、旋转角度

## 架构

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

## 开发状态

| 组件 | 状态 | 说明 |
|------|------|------|
| rotascope_app | ✅ 完成 | Flutter 显示端，JPEG 解码，触摸回传，陀螺仪切屏 |
| rotascope-server | ✅ MVP | 屏幕捕获 → WebSocket → JPEG 推流，15 FPS |
| rotascope_server2 | 🚧 开发中 | QUIC 传输 + H.264 编码（OpenH264）+ SendInput 注入 |
| rotascope-core | ✅ 完成 | 协议类型定义 |
| VirtualDisplayDriver | 🚧 开发中 | Windows IDD 驱动框架，支持 CMake 构建 |

详细说明见 [docs/usb-extended-display.md](docs/usb-extended-display.md)。
