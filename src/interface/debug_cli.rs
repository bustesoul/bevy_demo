//! 文字 CLI：读取 stdin → 解析命令 → 执行并打印

use bevy::app::AppExit;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::core::{events::LogEvent, states::AppState};
use crate::data::{ItemAssets, schema::ItemList};

static CLI_BUFFER: Lazy<Arc<Mutex<VecDeque<String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

/// 插件入口
pub struct DebugCliPlugin;
impl Plugin for DebugCliPlugin {
    fn build(&self, app: &mut App) {
        {
            let buffer = CLI_BUFFER.clone();
            std::thread::spawn(move || {
                use std::io::{self, BufRead};
                let stdin = io::stdin();
                for line_result in stdin.lock().lines() {
                    if let Ok(line) = line_result {
                        let line = line.trim();
                        if !line.is_empty() {
                            let mut buf = buffer.lock().unwrap();
                            buf.push_back(line.to_string());
                        }
                    }
                }
            });
        }
        app
            // 事件：原始输入行
            .add_event::<CliLine>()
            // 每帧从 buffer 取出所有命令行写入事件
            .add_systems(Update, read_stdin)
            // 仅在 InGame 处理命令
            .add_systems(
                Update,
                execute_cli_commands.run_if(in_state(AppState::InGame)),
            );
    }
}

/* ---------------------------- 事件与枚举 ---------------------------- */

/// 终端敲的一整行
#[derive(Event)]
struct CliLine(String);

/// 我们支持的命令
enum Command {
    Help,
    Status,
    Exit,
    Items(Option<String>), // None=全部；Some(token)=按 id/uuid/name 查询
    Give { id: String, count: u32 },
    Inventory,
    Equip { slot: String, index: usize },
    Unsupported(String),
}

/* ---------------------------- 读取 stdin ---------------------------- */

fn read_stdin(mut writer: EventWriter<CliLine>) {
    let mut buffer = CLI_BUFFER.lock().unwrap();
    while let Some(line) = buffer.pop_front() {
        writer.write(CliLine(line));
    }
}

/* ---------------------------- 命令执行 ---------------------------- */

fn execute_cli_commands(
    mut line_reader: EventReader<CliLine>,
    mut app_exit: EventWriter<AppExit>,
    mut log: EventWriter<LogEvent>,
    state: Res<State<AppState>>,
    item_assets: Res<ItemAssets>,
    lists: Res<Assets<ItemList>>,
    mut ev_give: EventWriter<crate::inventory::events::GiveItemEvent>,
    mut ev_list: EventWriter<crate::inventory::events::ListInventoryEvent>,
    mut ev_equip: EventWriter<crate::equipment::events::EquipEvent>,
) {
    for CliLine(input) in line_reader.read() {
        match parse_command(input) {
            Command::Help => {
                log.write(LogEvent(
                    "命令列表:
  help                   查看帮助
  status                 查看当前状态
  exit / quit            退出程序
  items                  列出所有物品
  items <token>          用 id / uuid / 名称 查询单个物品
  give <id> <count>      给予物品
  inventory              查看物品栏
  equip <slot> <index>   装备物品
  ".into()));
            }

            Command::Status => {
                let cnt = item_assets
                    .handle
                    .as_ref()
                    .and_then(|h| lists.get(h))
                    .map_or(0, |list| list.items.len());
                log.write(LogEvent(format!(
                    "State: {:?}, Items Loaded: {}",
                    state.get(),
                    cnt
                )));
            }

            Command::Exit => {
                log.write(LogEvent("Bye~".into()));
                app_exit.write(AppExit::Error(NonZero::<u8>::MIN));
            }

            Command::Items(token) => {
                if let Some(handle) = &item_assets.handle {
                    if let Some(list) = lists.get(handle) {
                        match token {
                            None => {
                                // 全部列出
                                for entry in &list.items {
                                    let uuid = uuid_from_id(&entry.id);
                                    log.write(LogEvent(format!(
                                        "{} | {} | {}",
                                        uuid, entry.id, entry.name
                                    )));
                                }
                            }
                            Some(t) => {
                                // 按三种字段模糊匹配
                                let t_low = t.to_lowercase();
                                if let Some(e) = list.items.iter().find(|e| {
                                    e.id.eq_ignore_ascii_case(&t_low)
                                        || e.name.eq_ignore_ascii_case(&t_low)
                                        || uuid_from_id(&e.id).to_string() == t_low
                                }) {
                                    let uuid = uuid_from_id(&e.id);
                                    log.write(LogEvent(format!(
                                        "==================================================
UUID : {uuid}
ID   : {}
Name : {}
Atk  : {}
Heal : {}
==================================================",
                                        e.id, e.name, e.atk, e.heal
                                    )));
                                } else {
                                    log.write(LogEvent("未找到匹配物品".into()));
                                }
                            }
                        }
                    }
                }
            }

            Command::Give { id, count } => {
                ev_give.write(crate::inventory::events::GiveItemEvent { id, count });
            }

            Command::Inventory => {
                ev_list.write(crate::inventory::events::ListInventoryEvent);
            }

            Command::Equip { slot, index } => {
                ev_equip.write(crate::equipment::events::EquipEvent { slot, index });
            }
            
            Command::Unsupported(cmd) => {
                log.write(LogEvent(format!("不支持的命令: {cmd}")));
            }
        }
    }
}

/* ---------------------------- 工具函数 ---------------------------- */

fn parse_command(input: &str) -> Command {
    let mut parts = input.split_whitespace();
    let cmd = parts.next().unwrap_or("").to_lowercase();
    match cmd.as_str() {
        "help" | "h" | "?" => Command::Help,
        "status" | "s" => Command::Status,
        "exit" | "quit" | "q" => Command::Exit,
        "items" | "item" | "i" => {
            let token = parts.next().map(|s| s.to_string());
            Command::Items(token)
        }
        "give" => {
            let id = parts.next().unwrap_or("").to_string();
            let cnt = parts.next().unwrap_or("1").parse().unwrap_or(1);
            Command::Give { id, count: cnt }
        }
        "inventory" | "inv" => Command::Inventory,
        "equip" => {
            let slot = parts.next().unwrap_or("").to_string();
            let idx = parts.next().unwrap_or("0").parse().unwrap_or(0);
            Command::Equip { slot, index: idx }
        }
        other => Command::Unsupported(other.into()),
    }
}

fn uuid_from_id(id: &str) -> Uuid {
    // 用固定 namespace + id 字节生成版本 5 UUID，保证可重复得到同一值
    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes())
}
