# rotascope_server2

RotaScope PC 服务端 v2，基于 QUIC 传输 + H.264 编码的高性能屏幕共享服务。

## 功能

- **QUIC 传输**：基于 quinn 的低延迟 QUIC 传输协议
- **H.264 编码**：使用 OpenH264 进行硬件加速视频编码
- **DXGI 屏幕捕获**：Windows Desktop Duplication API 高效捕获
- **SendInput 注入**：触摸事件转为鼠标输入注入
- **虚拟显示器**：通过 ChangeDisplaySettingsExW 创建虚拟显示器

## 快速开始

### 前置条件

- Rust 工具链（1.75+）
- Windows 10/11
- NVIDIA GPU（用于 NVENC 编码，可选）

### 运行

```bash
cargo run --release -- --listen-addr 0.0.0.0:1234
```
# 默认 60fps，JPEG 质量 40
```
.\start_usb_display.ps1
```
# 或手动调优（质量越高越清晰但帧率越低）
```
cargo run -- -Q 30 -d 0    # 质量 30 = 更高帧率
cargo run -- -Q 60 -d 0    # 质量 60 = 更清晰
```
质量与帧率的经验权衡：
• -Q 30: 约 50-60fps（最流畅）
• -Q 40: 约 40-55fps（默认平衡点）
• -Q 60: 约 30-40fps（画质优先
### 命令行参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--listen-addr` | `0.0.0.0:1234` | QUIC 服务器监听地址 |

## 项目结构

```
src/
├── main.rs              # 入口：clap 命令行参数 + QUIC 服务器启动
├── quic_server/
│   └── mod.rs           # QUIC 服务器（自签名证书、视频流推送）
├── capture/
│   └── mod.rs           # DXGI Desktop Duplication 屏幕捕获（Windows 专用）
├── encoder/
│   └── mod.rs           # OpenH264 编码器（BGRA/RGBA -> YUV420 -> H.264）
├── input_injector/
│   └── mod.rs           # SendInput 鼠标注入（Move/Down/Up）
├── virtual_display/
│   └── mod.rs           # 虚拟显示器管理（ChangeDisplaySettingsExW）
└── utils/
    ├── mod.rs
    └── utils.rs         # 工具函数
```

## 与 v1 的区别

| 特性 | v1 (rotascope-server) | v2 (rotascope_server2) |
|------|----------------------|----------------------|
| 传输协议 | WebSocket | QUIC |
| 视频编码 | JPEG | H.264 |
| 屏幕捕获 | scrap | DXGI Desktop Duplication |
| 帧率 | 15 FPS | 60 FPS |
| 触摸注入 | 仅日志 | SendInput 鼠标模拟 |

## 开发状态

- [x] QUIC 服务器
- [x] DXGI 屏幕捕获
- [x] OpenH264 编码器
- [x] SendInput 触摸注入
- [x] 虚拟显示器管理
- [ ] 音频传输
- [ ] 多显示器支持
- [ ] 性能优化