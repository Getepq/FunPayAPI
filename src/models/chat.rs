use serde::{Deserialize, Serialize};

// todo! Описать структуру превью чата.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatPreview {
    
}

// todo! Описать структуру полного чата.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chat {
    
}

// todo! Описать структуру cообщения.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    
}


#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsgFrom {
    User,
    System,
}

// todo! Описать все типы сообщений от пользователя.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsgTypes {
    Text,
    Image,
}

