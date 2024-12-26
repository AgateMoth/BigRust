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

#[derive(Serialize,Deserialize)]
struct SaveData{
    local_port: u16,
}

#[derive(Debug, PartialEq, Eq)]
struct User {
    user_name: String,
    user_password: String,
}



fn query_map_mysql(mut conn:PooledConn) ->Vec<User>{
    match conn.query_map(
        "SELECT user_name, user_password FROM users",
        |(user_name, user_password)|{
            User {user_name, user_password}
        }
    ){
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

fn insert_mysql(mut conn:PooledConn, user_name: &str, user_password: &str){
    let user=&vec![User{user_name: user_name.to_string(), user_password: user_password.to_string()}];
    match conn.exec_batch(
        r"INSERT INTO users (user_name, user_password)
          VALUES (:user_name, :user_password)",
          user.iter().map(|p: &User| params! {
            "user_name" => &p.user_name,
            "user_password" => &p.user_password,
        })
    )
    {
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
    password: &str) -> Result<bool, String> {
    // 连接到 MySQL 数据库
    let mysqlurl=format!("mysql://{}:{}@{}:{}/{}",mysqlname,mysqlpassword,mysqlhost,mysqlport,mysqldb);
    let pool = Pool::new(mysqlurl.as_str()).expect("Failed to create pool"); 
    let mut conn = pool.get_conn().expect("Failed to get connection from pool");
    let rows = query_map_mysql(conn);

    for row in rows {
        if row.user_name.as_str() == username && row.user_password.as_str() == password {
            return Ok(true);
        }
    }
    return Ok(false);
}
//createuser_in_mysql("root", "{密码}", "127.0.0.1", "3306", "testdatabase", "abcd", "1243");
#[tauri::command]
fn createuser_in_mysql(
    mysqlname: &str, 
    mysqlpassword: &str, 
    mysqlhost: &str, 
    mysqlport: &str, 
    mysqldb: &str, 
    username: &str,  
    password: &str) -> Result<(), String> {
    let mysqlurl=format!("mysql://{}:{}@{}:{}/{}",mysqlname,mysqlpassword,mysqlhost,mysqlport,mysqldb);
    let pool = Pool::new(mysqlurl.as_str()).expect("Failed to create pool"); 
    let mut conn = pool.get_conn().expect("Failed to get connection from pool");
    insert_mysql(conn, username, password);
    Ok(())
}


#[tauri::command]
fn set_local_port(port: u16, state: State<AppState>) -> Result<(), String> {
    let mut local_port = state.local_port.lock().map_err(|e| e.to_string())?;
    *local_port = port;
    set_local_port_savedfile(port).unwrap();
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

fn get_saved_data() -> SaveData {
    // 从文件中读取保存的数据
    // 返回一个 SaveData 结构体
    let savedfilepath="saved_data.json";
    if !Path::new(savedfilepath).exists() {
        let data: SaveData = SaveData {
            local_port: 8080,
        };
        match serde_json::to_string(&data) {
            Ok(serialized) => {
                match File::create(savedfilepath) {
                    Ok(mut file) => {
                        file.write_all(serialized.as_bytes()).expect("Failed to write to file");
                    }
                    Err(e) => {
                        println!("Failed to create file: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to serialize data: {}", e);
            }
        }
        return data;
    }
    else{
        let mut contents = String::new();
        match File::open(savedfilepath) {
            Ok(mut file) => {
                file.read_to_string(&mut contents).expect("Failed to read file");
            }
            Err(e) => {
                println!("Failed to open file: {}", e);
            }
        }
        match serde_json::from_str(&contents) {
            Ok(data) => {
                return data;
            }
            Err(e) => {
                println!("Failed to deserialize data: {}", e);
                return SaveData {
                    local_port: 8080,
                };
            }
        }
    }
}

fn set_local_port_savedfile(new_port: u16) -> Result<(), String> {
    let savedfilepath = "saved_data.json";
    if !Path::new(savedfilepath).exists() {
        return Err("File does not exist.".to_string());
    }

    let mut file = match OpenOptions::new().read(true).write(true).open(savedfilepath) {
        Ok(file) => file,
        Err(e) => return Err(format!("Failed to open file: {}", e)),
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
        return Err(format!("Failed to read file: {}", e));
    }

    let mut data: SaveData = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to deserialize data: {}", e)),
    };

    data.local_port = new_port;

    let serialized = match serde_json::to_string(&data) {
        Ok(serialized) => serialized,
        Err(e) => return Err(format!("Failed to serialize data: {}", e)),
    };

    if let Err(e) = file.set_len(0) {
        return Err(format!("Failed to truncate file: {}", e));
    }

    if let Err(e) = file.seek(std::io::SeekFrom::Start(0)) {
        return Err(format!("Failed to seek file: {}", e));
    }

    if let Err(e) = file.write_all(serialized.as_bytes()) {
        return Err(format!("Failed to write to file: {}", e));
    }

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


            let mut local_port = state.local_port.lock().map_err(|e| e.to_string())?;
            *local_port = get_saved_data().local_port;
            drop(local_port);

            let local_port = {
                let port = state.local_port.lock().unwrap();
                *port
            };

            thread::spawn(move || {
                let socket = UdpSocket::bind(format!("0.0.0.0:{}", local_port)).expect("Failed to bind socket");
                // println!("Listening on port {}",local_port);
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