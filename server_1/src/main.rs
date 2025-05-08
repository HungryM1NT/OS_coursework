use common::{FirstServerRequest, FirstServerResponse, MemoryUnit};
use sysinfo::System;
use std::net::{TcpListener, TcpStream};
use std::time::SystemTime;
use std::{process, thread};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use nix::unistd::{gethostname, getuid, User};


// Обработка пользовательских запросов
fn handle_client(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(size) => {
                let request: FirstServerRequest = match serde_json::from_slice(&buffer[..size]) {
                    Ok(request) => request,
                    Err(_) => {
                        return;
                    }
                };

                let mut system = System::new();
                system.refresh_memory();

                // Получение объем свободной памяти в задаваемых единицах
                let free_memory = match request.unit {
                    MemoryUnit::Bytes => system.free_memory() as f64,
                    MemoryUnit::MegaBytes => system.free_memory() as f64 / 1024.0 / 1024.0,
                    MemoryUnit::GigaBytes => system.free_memory() as f64 / 1024.0 / 1024.0 / 1024.0
                };

                // Получение имени компьютера и имени пользователя
                let hostname = gethostname().unwrap().into_string().unwrap();
                let username = User::from_uid(getuid()).unwrap().unwrap().name;

                // Получение времени
                let time_in_sec= SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() + 3 * 3600;
                let h = time_in_sec % 86400 / 3600;
                let m = time_in_sec % 3600 / 60;
                let s = time_in_sec % 60;
                let time = format!("{:.2}:{:.2}:{:.2}", h, m, s);

                // Формирование ответа
                let response = FirstServerResponse {
                    hostname,
                    username,
                    free_memory,
                    unit: match request.unit {
                        MemoryUnit::Bytes => String::from("bytes"),
                        MemoryUnit::MegaBytes => String::from("MB"),
                        MemoryUnit::GigaBytes => String::from("GB"),            
                    },
                    timestamp: time
                };

                let response_json = serde_json::to_vec(&response).unwrap();
                stream.write_all(&response_json).unwrap();
            },
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
            }
        }
    } 
}

// Проверка на существование сервера
fn check_singleton(port: u16) {
    if TcpStream::connect(format!("0.0.0.0:{}", port)).is_ok() {
        eprintln!("Server 1 is alreay running on port {}", port);
        process::exit(1);
    }
}

fn main() {
    let port = 8080;
    check_singleton(port);

    // Связывание сокета
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to port {}: {}", port, e);
            process::exit(1);
        }
    };

    println!("Server 1 running on port {}", port);

    let listener = Arc::new(Mutex::new(listener));

    // Создание пяти потоков для обработки нескольких пользователей
    for _ in 0..5 {
        let listener = Arc::clone(&listener);
        thread::spawn(move || loop {
            let listener = listener.lock().unwrap();
            match listener.accept() {
                Ok((stream, _)) => {
                    thread::spawn(move || handle_client(stream));
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        });
    }
    
    // Блокировка основного потока для постоянной работы сервера
    loop {
        thread::park();
    }
}