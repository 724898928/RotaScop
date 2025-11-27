use crate::virtual_display::VirtualDisplayManager;
use futures::{SinkExt, StreamExt};
use rotascope_core::{
    ClientMessage, ServerMessage, SwitchDirection, deserialize_message, serialize_message,
};
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_tungstenite::accept_async;
use tungstenite::{Message, Utf8Bytes};
use crate::{CrossPlatformCapturer::CrossPlatformCapturer};
use rotascope_core::Result;
use serde_json;
use tokio::time::interval;
use crate::CrossPlatformCapturer::compress_frame;

use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct MultiDisplayServer {
    virtual_displays: Arc<VirtualDisplayManager>,
    current_display: Arc<RwLock<u8>>,
    clients: Arc<Mutex<Vec<tokio::sync::mpsc::Sender<ServerMessage>>>>,
}

impl MultiDisplayServer {
    pub fn new(display_count: u8) -> Result<Self> {
      //  let capturer = Arc::new(ScreenCapturer::new()?);

        let virtual_displays = Arc::new(VirtualDisplayManager::new(vec![
            (0, 1920, 1080),
            (1, 1920, 1080),
            (2, 2560, 1440),
        ])?);
        let current_display = Arc::new(RwLock::new(0));
        let clients = Arc::new(Mutex::new(Vec::new()));

        Ok(Self {
            virtual_displays,
            current_display,
            clients,
        })
    }

    pub async fn start_virtual_displays(&self) -> Result<()> {
        self.virtual_displays.initialize().await?;
        log::info!("Virtual displays initialized");
        println!("Virtual displays initialized");
        Ok(())
    }

    pub async fn start_server(&self, addr: &str) -> Result<()> {
        log::info!("Server listening on {}", addr);
        println!("Server listening on {}", addr);

        // 使用 owned clone 放入 Arc，使其可以安全地移动到后台任务中
        let server_arc = Arc::new(self.clone());
        // 启动屏幕捕获和流媒体任务
        let stream_arc = server_arc.clone();
       let t1 = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let capturer = CrossPlatformCapturer::new_primary().unwrap();
                stream_arc.start_streaming(capturer).await.unwrap()
            })});
        let addr_owned = addr.to_string();
        let s = std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let listener = TcpListener::bind(&addr_owned).await.unwrap();
                loop {
                    let (socket, addr) = listener.accept().await.unwrap();
                    log::info!("New client connected: {}", addr);
                    println!("New client connected: {}", addr);

                    let client_arc = server_arc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = client_arc.handle_client(socket).await {
                            log::error!("Client handling error: {}", e);
                            println!("Client handling error: {}", e);
                        }
                    });
                }
            });
        });
       // t1.join().unwrap();
        s.join().unwrap();
        Ok(())
    }

    async fn handle_client(&self, mut stream: TcpStream) -> Result<()> {
        println!("handle_client");
        let ws_stream = accept_async(stream).await.expect("Failed to accept");

        let (mut writer, reader) = ws_stream.split();

        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

        // 添加到客户端列表
        {
            let mut clients = self.clients.lock().await;
            clients.push(tx.clone());
        }

        self.send_config_to_client(&mut writer).await?;
        // 处理来自客户端的消息
        let receive_task = self.deal_msg_from_client(reader);

        let send_task = Self::send_msg2client(writer, rx);
        // 等待任一任务完成
        tokio::select! {
            _ = receive_task => {},
            _ = send_task => {},
        }

        // 从客户端列表移除（通过释放 tx 的克隆来标记该客户端已断开）
        {
            let mut clients = self.clients.lock().await;
            clients.retain(|client_tx| !client_tx.is_closed());
        }

        // 显式关闭当前客户端的发送端
        drop(tx);

        Ok(())
    }

    fn send_msg2client(
        mut writer: futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<TcpStream>,
            tokio_tungstenite::tungstenite::Message,
        >,
        mut rx: Receiver<ServerMessage>,
    ) -> JoinHandle<()> {
        // 发送视频流到客户端
        let send_task = tokio::spawn(async move {
            println!("send_msg2client send_task");
            while let Some(message) = rx.recv().await {
                println!("send_msg2client send_task recv message ");
                let msg = serialize_message(&message).unwrap();
               // println!("send_msg2client msg:{:?}",message );
                match message {
                    ServerMessage::VideoFrame { data, .. } => {
                        println!("send_msg2client message data.len():{:?}",data.len() );
                        if let Err(e) = writer.send(Message::binary(data)).await {
                            log::error!("Error sending binary frame: {}", e);
                            // break;
                        }
                    }
                    other_message => {
                        if let Ok(text) = serialize_message(&other_message) {
                            if let Err(e) = writer.send(Message::Text(Utf8Bytes::try_from(text).unwrap())).await {
                                log::error!("Error sending text message: {}", e);
                            //    break;
                            }
                        }
                    }
                }
            }
        });
        send_task
    }

    fn deal_msg_from_client(
        &self,
        reader: futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<TcpStream>>,
    ) -> JoinHandle<()> {
        let client_arc = self.clone();
        let receive_task = tokio::spawn(async move {
            println!("deal_msg_from_client receive_task");
            let mut read_stream = reader;
            while let Some(result) = read_stream.next().await {
                println!("deal_msg_from_client message: {:?}", &result);
                match result {
                    std::result::Result::Ok(msg) => {
                        println!("deal_msg_from_client msg: {:?}", &msg);
                        match msg {
                            Message::Text(text) => {}
                            Message::Binary(data) => {
                                if let Ok(client_msg) =
                                    deserialize_message::<ClientMessage>(&data)
                                {
                                    if let Err(e) =
                                        client_arc.handle_client_message(client_msg).await
                                    {
                                        log::error!("Error handling client message: {}", e);
                                        println!("Error handling client message: {}", e);
                                    }
                                }
                            }
                            Message::Close(_) => {
                                println!("Client  sent close frame");
                                break;
                            }
                            Message::Ping(ping_data) => {
                                // 自动响应 pong
                            }
                            Message::Pong(_) => {
                                // 忽略 pong 响应
                            }
                            _ => {}
                        }
                    }
                    std::result::Result::Err(e) => {
                        log::error!("Error reading from client: {}", e);
                        println!("Error reading from client: {}", e);
                        break;
                    }
                }
            }
        });
        receive_task
    }

    async fn send_config_to_client(
        &self,
        writer: &mut futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<TcpStream>,
            tokio_tungstenite::tungstenite::Message,
        >,
    ) -> Result<()> {
        println!("send_config_to_client");
        // 发送初始配置
        let config = ServerMessage::DisplayConfig {
            total_displays: self.virtual_displays.get_display_count(),
            current_display: *self.current_display.read().await,
            resolutions: vec![(1920, 1080); 3], // 示例分辨率
        };

        let config_data = serialize_message(&config)?;
        writer
            .send(Message::Binary(
                config_data.into(),
            ))
            .await.map_err(|e|e.to_string())?;
        Ok(())
    }

    async fn handle_client_message(&self, message: ClientMessage) -> Result<()> {
        println!("handle_client_message");
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
        println!("switch_display");
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
        println!("Switched to display {}", *current);
        Ok(())
    }

    async fn start_streaming(&self, mut capturer:CrossPlatformCapturer) -> Result<()>{
        println!("start_streaming");
       // let mut interval = interval(Duration::from_millis(33)); // ~30fps
        loop {
          //  interval.tick().await;
            let current_display = *self.current_display.read().await;
            match capturer.capture_frame() {
                Ok(frame_data) => {
                    println!("start_streaming frame_data");
                    let message = ServerMessage::VideoFrame {
                        display_index: current_display,
                        width: frame_data.width(),
                        height: frame_data.height(),
                        data: compress_frame(&frame_data)?.to_vec(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    };
                   // println!("start_streaming frame_data data.len():{:?}", frame_data.to_vec().len() );
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
                    println!("Capture error: {}", e);
                }
            }
        }
        Ok(())
    }
}
