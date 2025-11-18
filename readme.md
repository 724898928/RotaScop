# RotaScope

## 产品概念简述

    目标：
        将一台安卓手机变成一个“虚拟多屏头显设备”，可作为个人便携式开发/工作环境使用。
        用户戴上头显（或将手机固定在眼前），通过分屏显示 + 体感或蓝牙外设交互，实现：

        左右眼双屏视觉（VR-like）
        多窗口编程、调试、笔记、终端环境
        连接外部键盘鼠标
        甚至通过云端 Rust 服务端获取计算能力

## 项目结构
```
rotascope/                    # 主项目目录
├── rotascope-server/         # Rust PC服务端
├── rotascope-app/           # Flutter移动端  
├── rotascope-core/          # 共享核心库
├── docs/                    # 文档
└── scripts/                 # 构建脚本

```
