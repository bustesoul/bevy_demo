use bevy::prelude::*;
use crate::core::events::LogEvent;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::VecDeque;
use once_cell::sync::Lazy;

static CLI_BUFFER: Lazy<Arc<Mutex<VecDeque<String>>>> = Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

pub struct DebugCliPlugin;

impl Plugin for DebugCliPlugin {
    fn build(&self, app: &mut App) {
        // 启动一次输入线程
        let cli_buffer = CLI_BUFFER.clone();
        thread::spawn(move || {
            let stdin = std::io::stdin();
            loop {
                let mut buf = String::new();
                if stdin.read_line(&mut buf).is_ok() {
                    let input = buf.trim().to_string();
                    if !input.is_empty() {
                        let mut buffer = cli_buffer.lock().unwrap();
                        buffer.push_back(input);
                    }
                }
            }
        });

        app.add_systems(Update, poll_cli_buffer);
    }
}

/// 每帧从缓冲区尝试弹出一行并写事件
fn poll_cli_buffer(mut writer: EventWriter<LogEvent>) {
    let mut buffer = CLI_BUFFER.lock().unwrap();
    while let Some(line) = buffer.pop_front() {
        writer.send(LogEvent(line));
    }
}