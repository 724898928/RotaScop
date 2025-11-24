mod virtual_display;
mod capture;
mod server;

use actix_web::{web, App, HttpServer};
use server::MultiDisplayServer;
use std::sync::Arc;
use anyhow::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();

    log::info!("Starting Rotascope Server with Actix-web...");

    // 创建多显示器服务器实例
    let display_server = Arc::new(MultiDisplayServer::new(1).await?);

    // 克隆 Arc 用于 HTTP 服务器
    let server_data = web::Data::new(display_server.clone());

    // 启动屏幕捕获和流媒体任务
    let stream_server = display_server.clone();
    actix_rt::spawn(async move {
        if let Err(e) = stream_server.start_streaming().await {
            log::error!("Streaming task error: {}", e);
        }
    });

    log::info!("Starting HTTP server on 0.0.0.0:8080");

    // 启动 HTTP 服务器
    HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .service(web::resource("/ws").to(websocket_handler))
            .service(actix_web::web::resource("/").to(|| async {
                actix_web::HttpResponse::Ok().body(
                    r#"
                    <html>
                        <head><title>Rotascope Server</title></head>
                        <body>
                            <h1>Rotascope Server is Running</h1>
                            <p>Connect with the mobile app via WebSocket.</p>
                            <script>
                                const ws = new WebSocket('ws://' + window.location.host + '/ws');
                                ws.onopen = () => console.log('Connected');
                                ws.onmessage = (event) => console.log('Message:', event.data);
                            </script>
                        </body>
                    </html>
                    "#
                )
            }))
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await?;

    Ok(())
}

// WebSocket 处理器
async fn websocket_handler(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    server: web::Data<Arc<MultiDisplayServer>>,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    use actix_web_actors::ws;

    let ws_actor = server::WebSocketActor::new(server.get_ref().clone());
    ws::start(ws_actor, &req, stream)
}