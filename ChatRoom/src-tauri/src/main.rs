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
    file_receive_port: Mutex<u16> // 文件接收端口
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

#[derive(Serialize, Deserialize, Clone)]
struct FileMeta {
    filename: String,
    total_size: u64,
    chunk_size: usize,
    total_chunks: u64,
}

#[derive(Serialize, Deserialize)]
struct FileChunk {
    sequence_number: u64,
    data: Vec<u8>,
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

// 设置文件线程端口 避免和消息线程冲突
#[tauri::command]
fn set_file_receive_port(port: u16, state: State<AppState>) -> Result<(), String> {
    let mut file_receive_port = state.file_receive_port.lock().map_err(|e| e.to_string())?;
    *file_receive_port = port;
    Ok(())
}

#[tauri::command]
fn get_file_receive_port(state: State<AppState>) -> Result<u16, String> {
    let file_receive_port = state.file_receive_port.lock().map_err(|e| e.to_string())?;
    Ok(*file_receive_port)
}


#[tauri::command]
fn send_file(
    file_path: String,
    target: String,
    port: u16,
) -> Result<(), String> {
    println!("开始发送文件: {}, 目标: {}, 端口: {}", file_path, target, port);

    // 创建 UDP 套接字
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| {
        let err_msg = format!("无法绑定 UDP 套接字: {}", e);
        println!("{}", err_msg);
        err_msg
    })?;
    let target_addr = format!("{}:{}", target, port);

    // 打开文件
    let mut file = File::open(&file_path).map_err(|e| {
        let err_msg = format!("无法打开文件 {}: {}", file_path, e);
        println!("{}", err_msg);
        err_msg
    })?;
    let metadata = file.metadata().map_err(|e| {
        let err_msg = format!("无法获取文件元信息 {}: {}", file_path, e);
        println!("{}", err_msg);
        err_msg
    })?;
    let total_size = metadata.len();
    let chunk_size = 1024;
    let total_chunks = (total_size + chunk_size as u64 - 1) / chunk_size as u64;

    println!(
        "文件元信息: 文件名: {}, 总大小: {}, 每块大小: {}, 总块数: {}",
        file_path, total_size, chunk_size, total_chunks
    );

    // 发送文件元信息
    let meta = FileMeta {
        filename: file_path.clone(),
        total_size,
        chunk_size,
        total_chunks,
    };
    let serialized_meta = serde_json::to_string(&meta).map_err(|e| {
        let err_msg = format!("无法序列化文件元信息: {}", e);
        println!("{}", err_msg);
        err_msg
    })?;
    socket.send_to(serialized_meta.as_bytes(), &target_addr).map_err(|e| {
        let err_msg = format!("发送文件元信息失败: {}", e);
        println!("{}", err_msg);
        err_msg
    })?;
    println!("文件元信息已发送: {}", serialized_meta);

    // 逐块发送文件
    let mut buffer = vec![0u8; chunk_size];
    for sequence_number in 0..total_chunks {
        let bytes_read = file.read(&mut buffer).map_err(|e| {
            let err_msg = format!("读取文件块失败: 序号: {}/{}，错误: {}", sequence_number + 1, total_chunks, e);
            println!("{}", err_msg);
            err_msg
        })?;

        let chunk = FileChunk {
            sequence_number,
            data: buffer[..bytes_read].to_vec(),
        };
        let serialized_chunk = serde_json::to_string(&chunk).map_err(|e| {
            let err_msg = format!("无法序列化文件块: 序号: {}/{}，错误: {}", sequence_number + 1, total_chunks, e);
            println!("{}", err_msg);
            err_msg
        })?;
        socket.send_to(serialized_chunk.as_bytes(), &target_addr).map_err(|e| {
            let err_msg = format!("发送文件块失败: 序号: {}/{}，错误: {}", sequence_number + 1, total_chunks, e);
            println!("{}", err_msg);
            err_msg
        })?;
        println!("文件块已发送: 序号: {}/{}, 大小: {} 字节", sequence_number + 1, total_chunks, bytes_read);

        // TODO: 在这里实现 ACK 机制
    }

    println!("文件发送完成: 文件名: {}", file_path);
    Ok(())
}




fn receive_file_nonblocking(socket: &UdpSocket) -> Result<(), std::io::Error> {
    let mut buf = [0; 2048];
    let mut file_meta: Option<FileMeta> = None;
    let mut received_chunks = std::collections::HashMap::new();

    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            let msg = std::str::from_utf8(&buf[..amt]).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
            if let Some(meta) = &file_meta {
                // 接收文件块
                if let Ok(chunk) = serde_json::from_str::<FileChunk>(msg) {
                    received_chunks.insert(chunk.sequence_number, chunk.data);
                    println!("接收到文件块: 序号: {}/{}, 来源: {}", chunk.sequence_number + 1, meta.total_chunks, src);

                    // 发送 ACK
                    socket.send_to(&chunk.sequence_number.to_le_bytes(), &src)?;

                    // 判断是否完成
                    if received_chunks.len() as u64 == meta.total_chunks {
                        println!("文件接收完成: 文件名: {}", meta.filename);
                        let mut file = File::create(&meta.filename)?;
                        for seq in 0..meta.total_chunks {
                            if let Some(data) = received_chunks.get(&seq) {
                                file.write_all(data)?;
                            }
                        }
                        return Ok(()); // 文件接收完成
                    }
                }
            } else if let Ok(meta) = serde_json::from_str::<FileMeta>(msg) {
                // 接收文件元信息
                file_meta = Some(meta.clone());
                println!("接收到文件元信息: 文件名: {}, 总大小: {}, 总块数: {}", meta.filename, meta.total_size, meta.total_chunks);
            }
        }
        Err(e) => return Err(e),
    }

    Ok(())
}



fn main() {
    tauri::Builder::default()
        .manage(AppState {
            local_port: Mutex::new(8080),
            file_receive_port: Mutex::new(8081), // 初始化文件接收端口
        })
        .invoke_handler(tauri::generate_handler![
            send_message,
            set_local_port,
            get_local_port,
            find_in_mysql,
            createuser_in_mysql,
            send_file,
            set_file_receive_port,
            get_file_receive_port
        ])
        .setup(|app| {
            println!("启动！");
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

            // 初始化文件接收端口
            let mut file_receive_port = state.file_receive_port.lock().unwrap();
            *file_receive_port = 8081; // 默认端口
            drop(file_receive_port);

             // 启动文件接收线程
            let file_receive_port = {
                let port = state.file_receive_port.lock().unwrap();
                *port
            };

            thread::spawn(move || {
                let socket = UdpSocket::bind(format!("0.0.0.0:{}", file_receive_port))
                    .expect("Failed to bind file receive socket");
                socket
                    .set_nonblocking(true)
                    .expect("Failed to set socket to non-blocking mode");
                println!("文件接收线程已启动，监听端口: {}", file_receive_port);

                loop {
                    match receive_file_nonblocking(&socket) {
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // 没有数据到达，继续循环
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        Err(e) => {
                            println!("文件接收失败: {}", e);
                            break;
                        }
                    }
                }
            });




            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
