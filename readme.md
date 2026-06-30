# RotaScope

RotaScope 旨在把 Android 手机作为电脑的 USB 扩展显示器使用。

当前代码提供一个可运行的 MVP：

- `rotascope_app/`：Flutter Android 显示端，默认连接 USB 反向端口 `127.0.0.1:8083/ws`。
- `rotascope-server/`：Rust PC 服务端，捕获指定显示器并通过 WebSocket 推送 JPEG 帧。
- `scripts/start_usb_display.ps1`：Windows 启动脚本，自动配置 `adb reverse` 并启动服务端。
- `VirtualDisplayDriver/`：Windows 虚拟显示驱动雏形，后续用于让系统真正出现一块可扩展桌面的显示器。

快速启动：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start_usb_display.ps1
```

然后在 Android 设备上运行 Flutter App：

```powershell
cd .\rotascope_app
flutter run -d <android-device-id>
```

详细说明见 [docs/usb-extended-display.md](docs/usb-extended-display.md)。

注意：Android App 本身不能让 Windows 新增显示器。真正的扩展屏需要 PC 端虚拟显示驱动安装成功后，再由 RotaScope 服务端捕获该虚拟显示器并推送到手机。

```
┌──────────────── WINDOWS ────────────────┐
│  IDD Virtual Display Driver (C++)      │
│            ↓                           │
│   DWM Render Target (GPU Surface)     │
│            ↓                           │
│   DXGI Capture (Zero Copy)            │
│            ↓                           │
│   NVENC Encoder (Low Latency)         │
│            ↓                           │
│   USB Transport (WinUSB / ADB)        │
│            ↓                           │
│   Input Controller (SendInput)        │
└───────────────┬────────────────────────┘
                │ USB 3.0
┌───────────────▼────────────────────────┐
│             ANDROID                    │
│  MediaCodec H264 Decoder              │
│            ↓                          │
│  SurfaceView / OpenGL Renderer        │
│            ↓                          │
│  Touch → Input Back Channel           │
└────────────────────────────────────────┘
```