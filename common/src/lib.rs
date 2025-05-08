use serde::{Deserialize, Serialize};

// Перечисление единиц памяти
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum MemoryUnit {
    Bytes,
    MegaBytes,
    GigaBytes,
}

// Структура запроса первому серверу
#[derive(Debug, Serialize, Deserialize)]
pub struct FirstServerRequest {
    pub unit: MemoryUnit,
}

// Структура ответа первого сервера
#[derive(Debug, Serialize, Deserialize)]
pub struct FirstServerResponse {
    pub hostname: String,
    pub username: String,
    pub free_memory: f64,
    pub unit: String,
    pub timestamp: String
}

// Структура запроса второму серверу
#[derive(Debug, Serialize, Deserialize)]
pub struct  SecondServerRequest {
    pub request: Option<()>,
}

// Структура ответа второго сервера
#[derive(Debug, Serialize, Deserialize)]
pub struct SecondServerResponse {
    pub priority: i32,
    pub thread_ids: Vec<u32>,
    pub timestamp: String
}

// Перечисление серверов
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerNum {
    Server1,
    Server2,
}

