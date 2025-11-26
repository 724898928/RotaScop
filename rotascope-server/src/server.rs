use crate::capture::ScreenCapturer;
use crate::virtual_display::VirtualDisplayManager;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use rotascope_core::{
    ClientMessage, ServerMessage, SwitchDirection, deserialize_message, serialize_message,
};
use std::sync::Arc;
use std::io::ErrorKind;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, interval};
use tokio_util::bytes;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::codec::{FramedRead, FramedWrite};
pub struct MultiDisplayServer {
    capturer: Arc<ScreenCapturer>,
    virtual_displays: Arc<VirtualDisplayManager>,
    current_display: Arc<RwLock<u8>>,
    clients: Arc<Mutex<Vec<tokio::sync::mpsc::Sender<ServerMessage>>>>,
}

impl MultiDisplayServer {
    pub fn new(display_count: u8) -> Result<Self> {
        let capturer = Arc::new(ScreenCapturer::new()?);
        let virtual_displays = Arc::new(VirtualDisplayManager::new(vec![
            (0, 1920, 1080),
            (1, 1920, 1080),
            (2, 2560, 1440),
        ])?);
        let current_display = Arc::new(RwLock::new(0));
        let clients = Arc::new(Mutex::new(Vec::new()));

        Ok(Self {
            capturer,
            virtual_displays,
            current_display,
            clients,
        })
    }

    pub async fn start_virtual_displays(&self) -> Result<()> {
        self.virtual_displays.initialize().await?;
        log::info!("Virtual displays initialized");
        Ok(())
    }

    pub async fn start_server(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        log::info!("Server listening on {}", addr);

        // 使用 owned clone 放入 Arc，使其可以安全地移动到后台任务中
        let server_arc = Arc::new(self.clone());
        // 启动屏幕捕获和流媒体任务
        let stream_arc = server_arc.clone();
        tokio::spawn(async move {
            stream_arc.start_streaming().await;
        });

        loop {
            let (socket, addr) = listener.accept().await?;
            log::info!("New client connected: {}", addr);

            let client_arc = server_arc.clone();
            tokio::spawn(async move {
                if let Err(e) = client_arc.handle_client(socket).await {
                    log::error!("Client handling error: {}", e);
                }
            });
        }
    }

    async fn handle_client(&self, mut stream: TcpStream) -> Result<()> {
        // try to reduce write-side aborts by disabling Nagle
        let _ = stream.set_nodelay(true);
        // 使用自定义配置的 LengthDelimitedCodec，增大最大帧大小以允许发送较大的视频帧
        let read_codec = LengthDelimitedCodec::builder()
            .length_field_length(4)
            // 根据项目帧大小需要调整，这里设置为 200MB，确保 client 端也使用相同的长度字段与大小限制
            .max_frame_length(200 * 1024 * 1024) // 200 MB
            .new_codec();
        let write_codec = read_codec.clone();

        let (r, w) = tokio::io::split(stream);
        let mut reader = FramedRead::new(r, read_codec);
        let mut writer = FramedWrite::new(w, write_codec);

        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // 添加到客户端列表
        {
            let mut clients = self.clients.lock().await;
            clients.push(tx);
        }

        // 发送初始配置
        let config = ServerMessage::DisplayConfig {
            total_displays: self.virtual_displays.get_display_count(),
            current_display: *self.current_display.read().await,
            resolutions: vec![(1920, 1080); 3], // 示例分辨率
        };

        let config_data = serialize_message(&config)?;
        writer.send(bytes::Bytes::from(config_data)).await?;

        // 处理来自客户端的消息
        let client_arc = self.clone();
        let receive_task = tokio::spawn(async move {
            while let Some(message) = reader.next().await {
                match message {
                    Ok(data) => {
                        if let Ok(client_msg) = deserialize_message::<ClientMessage>(&data) {
                            if let Err(e) = client_arc.handle_client_message(client_msg).await {
                                log::error!("Error handling client message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading from client: {}", e);
                        break;
                    }
                }
            }
        });

        // 发送视频流到客户端
        let send_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let data = match serialize_message(&message) {
                    Ok(data) => bytes::Bytes::from(data),
                    Err(e) => {
                        log::error!("Serialization error: {}", e);
                        continue;
                    }
                };

                if let Err(e) = writer.send(data).await {
                    // 对于常见的连接中断，降低日志级别并尝试优雅关闭 writer
                    match e.kind() {
                        ErrorKind::BrokenPipe | ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
                            log::info!("Client connection closed by peer: {}", e);
                        }
                        _ => {
                            log::error!("Error sending to client: {}", e);
                        }
                    }
                    // 尝试关闭 writer（吞掉可能的错误），然后退出发送任务以便上层清理客户端
                    let _ = writer.close().await;
                    break;
                }
            }
        });

        // 等待任一任务完成
        tokio::select! {
            _ = receive_task => {},
            _ = send_task => {},
        }

        // 从客户端列表移除
        {
            let mut clients = self.clients.lock().await;
            clients.retain(|client_tx| !client_tx.is_closed());
        }

        Ok(())
    }

    async fn handle_client_message(&self, message: ClientMessage) -> Result<()> {
        match message {
            ClientMessage::SensorData { rotation_y, .. } => {
                // 根据旋转数据切换显示器
                if rotation_y > 30.0 {
                    self.switch_display(SwitchDirection::Next).await?;
                } else if rotation_y < -30.0 {
                    self.switch_display(SwitchDirection::Previous).await?;
                }
            }
            ClientMessage::SwitchDisplay { direction } => {
                self.switch_display(direction).await?;
            }
            ClientMessage::Heartbeat => {
                // 心跳处理
            }
        }
        Ok(())
    }

    async fn switch_display(&self, direction: SwitchDirection) -> Result<()> {
        let total_displays = self.virtual_displays.get_display_count() as u8;
        let mut current = self.current_display.write().await;

        match direction {
            SwitchDirection::Next => {
                *current = (*current + 1) % total_displays;
            }
            SwitchDirection::Previous => {
                *current = if *current == 0 {
                    total_displays - 1
                } else {
                    *current - 1
                };
            }
        }

        log::info!("Switched to display {}", *current);
        Ok(())
    }

    async fn start_streaming(&self) {
        let mut interval = interval(Duration::from_millis(33)); // ~30fps

        loop {
            interval.tick().await;

            let current_display = *self.current_display.read().await;

            match self.capturer.capture_display(current_display).await {
                Ok(frame_data) => {
                    let message = ServerMessage::VideoFrame {
                        display_index: current_display,
                        width: frame_data.width,
                        height: frame_data.height,
                        data: frame_data.jpeg_data,
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    };

                    // 发送给所有连接的客户端
                    // 先克隆出当前的 Sender 列表并释放锁，避免在发送时持有锁
                    let clients_vec = {
                        let clients = self.clients.lock().await;
                        clients.clone()
                    };

                    for client in clients_vec.into_iter() {
                        if let Err(e) = client.send(message.clone()).await {
                            log::debug!("Failed to send to client (will cleanup): {}", e);
                            // 清理已关闭的客户端
                            let mut clients = self.clients.lock().await;
                            clients.retain(|c| !c.is_closed());
                        }
                    }
                }
                Err(e) => {
                    log::error!("Capture error: {}", e);
                }
            }
        }
    }
}

impl Clone for MultiDisplayServer {
    fn clone(&self) -> Self {
        Self {
            capturer: self.capturer.clone(),
            virtual_displays: self.virtual_displays.clone(),
            current_display: self.current_display.clone(),
            clients: self.clients.clone(),
        }
    }
}
