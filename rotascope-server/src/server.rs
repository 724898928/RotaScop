use crate::capture::ScreenCapturer;
use crate::virtual_display::VirtualDisplayManager;
use anyhow::Result;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};
use tokio_util::bytes;
use rotascope_core::{deserialize_message, serialize_message, ClientMessage, ServerMessage, SwitchDirection};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::{SinkExt, StreamExt};
pub struct MultiDisplayServer {
    capturer: Arc<ScreenCapturer>,
    virtual_displays: Arc<VirtualDisplayManager>,
    current_display: Arc<RwLock<u8>>,
    clients: Arc<Mutex<Vec<tokio::sync::mpsc::Sender<ServerMessage>>>>,
}

impl MultiDisplayServer {
    pub fn new(display_count: u8) -> Result<Self> {
        let capturer = Arc::new(ScreenCapturer::new()?);
        let virtual_displays = Arc::new(VirtualDisplayManager::new(display_count)?);
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

        let server_arc = Arc::new(self);

        // 启动屏幕捕获和流媒体任务
        let stream_arc = server_arc.clone();
        // 覆盖之前对 `self` 的引用，使用 owned clone 放入 Arc，使其可以安全地移动到后台任务中
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

    async fn handle_client(&self, stream: TcpStream) -> Result<()> {
        // 使用自定义配置的 LengthDelimitedCodec，增大最大帧大小以允许发送较大的视频帧
        let read_codec = LengthDelimitedCodec::builder()
            .length_field_length(4)
            .max_frame_length(100 * 1024 * 1024) // 50 MB
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
                    log::error!("Error sending to client: {}", e);
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
        let total_displays = self.virtual_displays.get_display_count();
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
                    let clients = self.clients.lock().await;
                    for client in clients.iter() {
                        if let Err(e) = client.send(message.clone()).await {
                            log::debug!("Failed to send to client: {}", e);
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