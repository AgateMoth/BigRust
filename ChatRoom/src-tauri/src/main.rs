use std::io::Seek;
use std::io::Write;
use std::net::UdpSocket;
use std::path::Path;
use std::thread;
use mysql::prelude::Queryable;
use serde::Deserialize;
use serde::Serialize;
use tauri::Manager;
use tauri::Emitter;
use tauri::State;
use std::sync::Mutex;
use std::fs::{File, OpenOptions};
use std::io::Read;
use serde_json;
use mysql::{Pool, PooledConn, params};

struct AppState {
    local_port: Mutex<u16>,
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    local_port: u16,
}

#[derive(Debug, PartialEq, Eq)]
struct User {
    user_name: String,
    user_password: String,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    username: String,
    content: String,
}

fn query_map_mysql(mut conn: PooledConn) -> Vec<User> {
    match conn.query_map(
        "SELECT user_name, user_password FROM users",
        |(user_name, user_password)| {
            User { user_name, user_password }
        },
    ) {
        Ok(rows) => {
            return rows;
        }
        Err(e) => {
            println!("Failed to query database: {}", e);
            println!("trying to create database");
            conn.query_drop(r"CREATE TABLE users(
                user_id int not null AUTO_INCREMENT,
                user_name varchar(255) not null,
                user_password varchar(255) not null,
                PRIMARY KEY (user_id)
            )").expect("Failed to create table");
            return query_map_mysql(conn);
        }
    }
}

fn insert_mysql(mut conn: PooledConn, user_name: &str, user_password: &str) {
    let user = &vec![User { user_name: user_name.to_string(), user_password: user_password.to_string() }];
    match conn.exec_batch(
        r"INSERT INTO users (user_name, user_password)
          VALUES (:user_name, :user_password)",
        user.iter().map(|p: &User| params! {
            "user_name" => &p.user_name,
            "user_password" => &p.user_password,
        }),
    ) {
        Ok(_) => {
            println!("Data inserted successfully");
        }
        Err(e) => {
            println!("Failed to insert database: {}", e);
            println!("trying to create database");
            conn.query_drop(r"CREATE TABLE users(
                user_id int not null AUTO_INCREMENT,
                user_name varchar(255) not null,
                user_password varchar(255) not null,
                PRIMARY KEY (user_id)
            )").expect("Failed to create table");
        }
    };
}

// find_in_mysql("root", "{密码}", "127.0.0.1", "3306", "testscheme", "abc", "123");
#[tauri::command]
fn find_in_mysql(
    mysqlname: &str, 
    mysqlpassword: &str, 
    mysqlhost: &str,
    mysqlport: &str, 
    mysqldb: &str,
    username: &str, 
    password: &str
) -> Result<bool, String> {
    // 连接到 MySQL 数据库
    let mysqlurl = format!("mysql://{}:{}@{}:{}/{}", mysqlname, mysqlpassword, mysqlhost, mysqlport, mysqldb);
    let pool = Pool::new(mysqlurl.as_str()).map_err(|e| e.to_string())?; 
    let conn = pool.get_conn().map_err(|e| e.to_string())?;
    let rows = query_map_mysql(conn);

    for row in rows {
        if row.user_name.as_str() == username && row.user_password.as_str() == password {
            return Ok(true);
        }
    }
    Ok(false)
}

// createuser_in_mysql("root", "{密码}", "127.0.0.1", "3306", "testdatabase", "abcd", "1243");
#[tauri::command]
fn createuser_in_mysql(
    mysqlname: &str, 
    mysqlpassword: &str, 
    mysqlhost: &str, 
    mysqlport: &str, 
    mysqldb: &str, 
    username: &str,  
    password: &str
) -> Result<(), String> {
    let mysqlurl = format!("mysql://{}:{}@{}:{}/{}", mysqlname, mysqlpassword, mysqlhost, mysqlport, mysqldb);
    let pool = Pool::new(mysqlurl.as_str()).map_err(|e| e.to_string())?; 
    let conn = pool.get_conn().map_err(|e| e.to_string())?;
    insert_mysql(conn, username, password);
    Ok(())
}

#[tauri::command]
fn set_local_port(port: u16, state: State<AppState>) -> Result<(), String> {
    let mut local_port = state.local_port.lock().map_err(|e| e.to_string())?;
    *local_port = port;
    set_local_port_savedfile(port).map_err(|e| e)?;
    Ok(())
}

#[tauri::command]
fn get_local_port(state: State<AppState>) -> Result<u16, String> {
    let local_port = state.local_port.lock().map_err(|e| e.to_string())?;
    Ok(*local_port)
}

#[tauri::command]
fn send_message(username: String, message: String, target: String, port: u16) -> Result<(), String> {
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    let target_addr = format!("{}:{}", target, port);
    let chat_msg = ChatMessage { username, content: message };
    let serialized = serde_json::to_string(&chat_msg).map_err(|e| e.to_string())?;
    socket.send_to(serialized.as_bytes(), target_addr).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_saved_data() -> SaveData {
    // 从文件中读取保存的数据
    // 返回一个 SaveData 结构体
    let savedfilepath = "saved_data.json";
    if !Path::new(savedfilepath).exists() {
        let data = SaveData {
            local_port: 8080,
        };
        if let Ok(serialized) = serde_json::to_string(&data) {
            if let Ok(mut file) = File::create(savedfilepath) {
                let _ = file.write_all(serialized.as_bytes());
            }
        }
        return data;
    } else {
        let mut contents = String::new();
        if let Ok(mut file) = File::open(savedfilepath) {
            let _ = file.read_to_string(&mut contents);
        }
        if let Ok(data) = serde_json::from_str(&contents) {
            return data;
        } else {
            println!("Failed to deserialize data");
            return SaveData {
                local_port: 8080,
            };
        }
    }
}

fn set_local_port_savedfile(new_port: u16) -> Result<(), String> {
    let savedfilepath = "saved_data.json";
    if !Path::new(savedfilepath).exists() {
        return Err("File does not exist.".to_string());
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(savedfilepath)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut data: SaveData = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to deserialize data: {}", e))?;

    data.local_port = new_port;

    let serialized = serde_json::to_string(&data)
        .map_err(|e| format!("Failed to serialize data: {}", e))?;

    file.set_len(0)
        .map_err(|e| format!("Failed to truncate file: {}", e))?;
    file.seek(std::io::SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek file: {}", e))?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| format!("Failed to write to file: {}", e))?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            local_port: Mutex::new(8080),
        })
        .invoke_handler(tauri::generate_handler![
            send_message, 
            set_local_port, 
            get_local_port,
            find_in_mysql,
            createuser_in_mysql
        ])
        .setup(|app| {
            let state = app.state::<AppState>();
            let app_handle = app.handle().clone();

            let mut local_port = state.local_port.lock().map_err(|e| e.to_string())?;
            *local_port = get_saved_data().local_port;
            drop(local_port);

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
                            if let Ok(msg_str) = std::str::from_utf8(&buf[..amt]) {
                                if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(msg_str) {
                                    let payload = serde_json::to_string(&chat_msg).unwrap_or_default();
                                    app_handle.emit("receive_message", payload).unwrap();
                                }
                            }
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