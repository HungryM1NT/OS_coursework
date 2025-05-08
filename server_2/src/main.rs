use common::{SecondServerRequest, SecondServerResponse};
use std::net::{TcpListener, TcpStream};
use std::time::SystemTime;
use std::{process, thread};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use procfs::process::Process;
use libc::{getpriority, PRIO_PROCESS};


// Обработка пользовательских запросов
fn handle_client(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(size) => {
                let request: SecondServerRequest = match serde_json::from_slice(&buffer[..size]) {
                    Ok(request) => request,
                    Err(_) => {
                        return;
                    }
                };

                // Получение приоритета серверного процесса
                let me = Process::myself().expect("Can't get Server 2 PID");
                let priority = unsafe { getpriority(PRIO_PROCESS, me.pid as u32) };

                // Получение идентификатора потоков процесса
                let mut thread_ids: Vec<u32> = Vec::new();
                if let Ok(tasks) = me.tasks() {
                    for task in tasks {
                        thread_ids.push(task.unwrap().tid as u32);
                    }
                }

                // Получение времени
                let time_in_sec= SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() + 3 * 3600;
                let h = time_in_sec % 86400 / 3600;
                let m = time_in_sec % 3600 / 60;
                let s = time_in_sec % 60;
                let time = format!("{:.2}:{:.2}:{:.2}", h, m, s);

                // Формирование ответа
                let response = SecondServerResponse {
                    priority,
                    thread_ids,
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
        eprintln!("Server 2 is alreay running on port {}", port);
        process::exit(1);
    }
}

fn main() {
    let port = 8081;
    check_singleton(port);

    // Связывание сокета
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to port {}: {}", port, e);
            process::exit(1);
        }
    };

    println!("Server 2 running on port {}", port);

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