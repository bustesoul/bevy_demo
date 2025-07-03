use bevy::prelude::*;

/// 仅作演示：打印一句话
#[derive(Event)]
pub struct LogEvent(pub String);

pub fn hello_world(mut writer: EventWriter<LogEvent>) {
    writer.write(LogEvent("Hello, Bevy!".into()));
}