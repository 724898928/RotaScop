use actix::{
    Actor, ActorContext, AsyncContext, Handler, Message, Recipient, StreamHandler,
    ActorFutureExt, ContextFutureSpawner, WrapFuture,
};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use crate::capture::ScreenCapturer;
use crate::virtual_display::VirtualDisplayManager;

// 客户端消息
#[derive(Deserialize, Debug)]
pub enum ClientMessage {
    SensorData {
        rotation_x: f32,
        rotation_y: f32,
        rotation_z: f32,
    },
    SwitchDisplay {
        direction: SwitchDirection,
    },
    Heartbeat,
}

#[derive(Deserialize, Debug)]
pub enum SwitchDirection {
    Next,
    Previous,
}

// 服务器消息
#[derive(Serialize, Debug, Clone)]
pub enum ServerMessage {
    VideoFrame {
        display_index: u8,
        width: u32,
        height: u32,
        data: Vec<u8>, // JPEG encoded
        timestamp: u64,
    },
    DisplayConfig {
        total_displays: u8,
        current_display: u8,
        resolutions: Vec<(u32, u32)>,
    },
    Heartbeat,
    Error {
        message: String,
    },
}

// WebSocket 会话消息
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub ServerMessage);

// 主服务器结构
pub struct MultiDisplayServer {
    pub capturer: Arc<ScreenCapturer>,
    pub virtual_displays: Arc<VirtualDisplayManager>,
    pub current_display: Arc<RwLock<u8>>,
    pub clients: Arc<RwLock<HashMap<usize, Recipient<BroadcastMessage>>>>,
    pub next_client_id: Arc<RwLock<usize>>,
}

impl MultiDisplayServer {
    pub async fn new(display_count: u8) -> Result<Self> {
        let capturer = Arc::new(ScreenCapturer::new()?);
        let virtual_displays = Arc::new(VirtualDisplayManager::new(display_count)?);

        // 初始化虚拟显示器
        virtual_displays.initialize().await?;

        let current_display = Arc::new(RwLock::new(0));
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let next_client_id = Arc::new(RwLock::new(0));

        Ok(Self {
            capturer,
            virtual_displays,
            current_display,
            clients,
            next_client_id,
        })
    }



    // 在 streaming 循环中添加性能监控
    pub async fn start_streaming(&self) -> Result<()> {
        use tokio::time::interval;
        let mut interval = interval(Duration::from_millis(100));

        let mut frame_count = 0;
        let mut total_size = 0;
        let mut total_capture_time = Duration::from_secs(0);
        let mut last_log = Instant::now();

        loop {
            interval.tick().await;

            let capture_start = Instant::now();
            let current_display = *self.current_display.read();

            match self.capturer.capture_display(current_display).await {
                Ok(frame_data) => {
                    let capture_time = capture_start.elapsed();

                    frame_count += 1;
                    total_size += frame_data.jpeg_data.len();
                    total_capture_time += capture_time;

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

                    // 定期记录性能统计
                    if last_log.elapsed() > Duration::from_secs(5) {
                        let avg_size = total_size / frame_count;
                        let avg_capture_time = total_capture_time / frame_count as u32;
                        log::info!("Streaming stats: {} frames, avg {} bytes/frame, avg capture time: {:?}",
                              frame_count, avg_size, avg_capture_time);

                        frame_count = 0;
                        total_size = 0;
                        total_capture_time = Duration::from_secs(0);
                        last_log = Instant::now();
                    }

                    // 广播给所有连接的客户端
                    self.broadcast_message(message).await;
                }
                Err(e) => {
                    log::error!("Capture error: {}", e);
                }
            }
        }
    }

    pub async fn broadcast_message(&self, message: ServerMessage) {
        let clients = self.clients.read().clone();

        for (client_id, recipient) in clients {
            let _ = recipient.do_send(BroadcastMessage(message.clone()));
        }
    }

    pub fn add_client(&self, recipient: Recipient<BroadcastMessage>) -> usize {
        let mut next_id = self.next_client_id.write();
        let client_id = *next_id;
        *next_id += 1;

        self.clients.write().insert(client_id, recipient.clone());

        // 发送当前配置给新客户端
        let config = ServerMessage::DisplayConfig {
            total_displays: self.virtual_displays.get_display_count(),
            current_display: *self.current_display.read(),
            resolutions: vec![(1920, 1080); 3],
        };

        let _ = recipient.do_send(BroadcastMessage(config));

        log::info!("New client connected: {}", client_id);
        client_id
    }

    pub fn remove_client(&self, client_id: usize) {
        self.clients.write().remove(&client_id);
        log::info!("Client disconnected: {}", client_id);
    }

    pub async fn handle_client_message(&self, message: ClientMessage) -> Result<()> {
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
                // 心跳响应
                self.broadcast_message(ServerMessage::Heartbeat).await;
            }
        }
        Ok(())
    }

    async fn switch_display(&self, direction: SwitchDirection) -> Result<()> {
        let total_displays = self.virtual_displays.get_display_count();
        let mut current = self.current_display.write();

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

        // 通知所有客户端显示配置已更新
        let config = ServerMessage::DisplayConfig {
            total_displays,
            current_display: *current,
            resolutions: vec![(1920, 1080); total_displays as usize],
        };

        self.broadcast_message(config).await;

        Ok(())
    }
}

// WebSocket Actor
pub struct WebSocketActor {
    server: Arc<MultiDisplayServer>,
    client_id: Option<usize>,
    hb: Instant,
}

impl WebSocketActor {
    pub fn new(server: Arc<MultiDisplayServer>) -> Self {
        Self {
            server,
            client_id: None,
            hb: Instant::now(),
        }
    }

    // 发送心跳
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
                log::warn!("Heartbeat timeout, disconnecting client");
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // 注册客户端到服务器
        let recipient = ctx.address().recipient();
        self.client_id = Some(self.server.add_client(recipient));

        // 开始心跳
        self.hb(ctx);

        log::info!("WebSocket connection established");
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        // 从服务器移除客户端
        if let Some(client_id) = self.client_id {
            self.server.remove_client(client_id);
        }

        actix::Running::Stop
    }
}

// 在 WebSocketActor 的 Handler<BroadcastMessage> 实现中修改：
impl Handler<BroadcastMessage> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        match msg.0 {
            ServerMessage::VideoFrame { data, .. } => {
                // 直接发送二进制数据，不进行 JSON 序列化
                ctx.binary(data);
            }
            other_message => {
                // 只有非视频帧消息才进行 JSON 序列化
                match serde_json::to_string(&other_message) {
                    Ok(json) => {
                        ctx.text(json);
                    }
                    Err(e) => {
                        log::error!("Failed to serialize message: {}", e);
                    }
                }
            }
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        let server = self.server.clone();
                        actix_rt::spawn(async move {
                            if let Err(e) = server.handle_client_message(client_msg).await {
                                log::error!("Error handling client message: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        log::warn!("Failed to parse client message: {}", e);

                        // 发送错误消息回客户端
                        let error_msg = ServerMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        };

                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            ctx.text(json);
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                log::warn!("Unexpected binary message from client: {} bytes", bin.len());
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}