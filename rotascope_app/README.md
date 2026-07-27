# rotascope_app

Flutter Android 显示端，用于将 Android 手机作为电脑的 USB 扩展显示器使用。

## 功能

- **全屏显示电脑画面**：通过 WebSocket 接收 JPEG 或 H.264 视频流，60 FPS
- **触摸回传**：手机触摸事件发送到电脑，支持鼠标模拟（v2）
- **陀螺仪切屏**：旋转手机切换显示器
- **自动重连**：指数退避重连机制
- **HUD 显示**：FPS、显示器编号、旋转角度、当前编码器
- **H.264 硬解码**：通过 Android MediaCodec API 硬件解码，降低带宽
- **QUIC 传输**：支持 QUIC 协议传输 H.264 视频流（需原生插件）

## 快速开始

### 前置条件

- Flutter SDK
- Android 设备或模拟器
- ADB 工具

### 运行

```bash
# 连接 Android 设备
adb devices

# 启动应用
flutter run -d <android-device-id>
```

### 连接配置

应用默认连接 `ws://127.0.0.1:8083/ws`（通过 `adb reverse` 映射到电脑）。

在电脑端执行：
```bash
adb reverse tcp:8083 tcp:8083
```

## 项目结构

```
lib/
├── main.dart              # 入口：Wakelock、横屏、沉浸模式
├── app.dart               # Material3 应用壳 + Provider 注入
├── model/
│   └── video_frame.dart   # 视频帧数据模型
├── screens/
│   └── remote_screen.dart # 主界面：HUD + 显示控件 + Sensor 事件
├── services/
│   ├── connection_service.dart      # WebSocket 连接、帧接收、自动重连
│   ├── sensor_service.dart          # 陀螺仪监听、旋转切屏
│   ├── h264_decoder_service.dart    # H.264 硬解码服务（MethodChannel → MediaCodec）
│   ├── quic_transport_service.dart  # QUIC 传输客户端（需原生插件）
│   └── video_pipeline_service.dart  # 视频管线：编解码切换 + 多传输后端
└── widgets/
    ├── display_view.dart        # JPEG 图片渲染 + 触摸手势
    ├── display_hud.dart         # HUD（FPS、显示器编号、旋转角度）
    └── connection_panel.dart    # 连接面板（地址输入、状态指示）
```

## 技术栈

- Flutter 3.x
- Dart
- web_socket_channel
- sensors_plus
- provider

## 开发状态

- [x] JPEG 流接收与渲染
- [x] 触摸事件回传
- [x] 陀螺仪切屏
- [x] 自动重连
- [x] HUD 显示
- [x] H.264 硬解码支持（v2）
- [x] QUIC 传输支持（v2，平台通道占位）