use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use futures::{future, lock::Mutex};

use actix_web::{App, HttpServer, Result, HttpResponse, error, http,
    web::{post, Data, Payload}};
use actix_web_static_files::ResourceFiles;
use webrtc_unreliable::{Server, SessionEndpoint};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listen_socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
    let public_socket = listen_socket;
    let webrtc_server = Server::new(listen_socket, public_socket).await?;    
    let http_server_fut = HttpServer::new({
        let se_mutex = Data::new(Mutex::new(webrtc_server.session_endpoint()));
        move || {
            let generated = generate();
            App::new()
                .app_data(se_mutex.clone())
                .route("/connect", post().to(connect))
                .service(ResourceFiles::new("", generated))
        }
    }).bind("127.0.0.1:8080")?.run();
    
    let recv_spin_fut = recv_spin(webrtc_server);
    future::try_join(http_server_fut, recv_spin_fut).await?;
    Ok(())
}

async fn connect(se_mutex: Data<Mutex<SessionEndpoint>>, sdp: Payload) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(se_mutex.lock().await
            .session_request(sdp)
            .await.map_err(error::ErrorBadRequest)?
    ))
}

async fn recv_spin(mut webrtc_server: Server) -> std::io::Result<()> {
    let mut message_buf: Vec<u8> = Vec::new();
    loop {
        let received = match webrtc_server.recv().await {
            Ok(received) => {
                message_buf.clear();
                message_buf.extend(received.message.as_ref());
                message_buf.append("cool".to_string().into_bytes().as_mut());
                Some((received.message_type, received.remote_addr))
            }
            Err(err) => {
                eprintln!("Could not receive RTC message: {}", err);
                None
            }
        };

        if let Some((message_type, remote_addr)) = received {
            if let Err(e) = webrtc_server
                .send(&message_buf, message_type, &remote_addr).await
            {
                eprintln!("Could not send message to {}: {}", remote_addr, e);
            }
        }
    }
}
