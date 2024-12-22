use std::net::UdpSocket;
use std::thread;
use tauri::Manager;
use tauri::State;
use std::sync::Mutex;

struct AppState {
    local_port: Mutex<u16>,
}

#[tauri::command]
fn set_local_port(port: u16, state: State<AppState>) -> Result<(), String> {
    let mut local_port = state.local_port.lock().map_err(|e| e.to_string())?;
    *local_port = port;
    Ok(())
}

#[tauri::command]
fn get_local_port(state: State<AppState>) -> Result<u16, String> {
    let local_port = state.local_port.lock().map_err(|e| e.to_string())?;
    Ok(*local_port)
}

#[tauri::command]
fn send_message(message: String, target: String, port: u16) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    let target_addr = format!("{}:{}", target, port);
    socket.send_to(message.as_bytes(), target_addr).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            local_port: Mutex::new(8080),
        })
        .invoke_handler(tauri::generate_handler![send_message, set_local_port, get_local_port])
        .setup(|app| {
            let state = app.state::<AppState>();
            let app_handle = app.handle().clone();

            let local_port = {
                let port = state.local_port.lock().unwrap();
                *port
            };

            thread::spawn(move || {
                let socket = UdpSocket::bind(format!("0.0.0.0:{}", local_port)).expect("Failed to bind socket");
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