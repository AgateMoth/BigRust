use std::net::UdpSocket;
use std::thread;
use tauri::Emitter;

fn main() {
    let _app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![send_message])
        .setup(|app| {
            let app_handle = app.handle().clone();
            thread::spawn(move || {
                let socket = UdpSocket::bind("0.0.0.0:8080").expect("Failed to bind socket");
                let mut buf = [0; 1024];
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((amt, _src)) => {
                            let msg = String::from_utf8_lossy(&buf[..amt]).to_string();
                            app_handle.emit("receive_message", msg).unwrap();
                        }
                        Err(_) => break,
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn send_message(message: String, target: String, port: u16) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    let target_addr = format!("{}:{}", target, port);
    socket.send_to(message.as_bytes(), target_addr).expect("Failed to send message");
}