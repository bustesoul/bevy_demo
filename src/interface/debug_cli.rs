//! 文字 CLI：读取 stdin → 解析命令 → 执行并打印

use bevy::app::AppExit;
use bevy::prelude::*;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::io::{self, Write};

use crate::core::{events::LogEvent, states::AppState};
use crate::data::{ItemAssets, schema::ItemList};
use crate::equipment::components::Equipment;
use crate::inventory::components::Backpack;
use crate::character::components::{Player, Stats};

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
                (execute_basic_commands, execute_character_commands, display_status_bar)
                    .run_if(in_state(AppState::InGame)),
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
    Unequip { slot: String },
    Use { index: usize },
    Stats,
    GainExp { amount: i32 },
    TakeDamage { damage: i32 },
    Heal { amount: i32 },
    Unsupported(String),
}

/* ---------------------------- 读取 stdin ---------------------------- */

static LAST_STATS_HASH: Lazy<Arc<Mutex<u64>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

fn read_stdin(mut writer: EventWriter<CliLine>) {
    let mut buffer = CLI_BUFFER.lock().unwrap();
    while let Some(line) = buffer.pop_front() {
        writer.write(CliLine(line));
    }
}

/* ---------------------------- 命令执行 ---------------------------- */

fn execute_basic_commands(
    mut line_reader: EventReader<CliLine>,
    mut app_exit: EventWriter<AppExit>,
    mut log: EventWriter<LogEvent>,
    state: Res<State<AppState>>,
    item_assets: Res<ItemAssets>,
    lists: Res<Assets<ItemList>>,
    backpack: Res<Backpack>,
    equipment: Res<Equipment>,
    player_query: Query<&Stats, With<Player>>,
    mut ev_give: EventWriter<crate::inventory::events::GiveItemEvent>,
    mut ev_list: EventWriter<crate::inventory::events::ListInventoryEvent>,
    mut ev_equip: EventWriter<crate::equipment::events::EquipEvent>,
    mut ev_unequip: EventWriter<crate::equipment::events::UnequipEvent>,
    mut ev_use: EventWriter<crate::inventory::events::UseItemEvent>,
) {
    for CliLine(input) in line_reader.read() {
        // 在处理命令前先打印换行，清除状态栏
        println!();
        
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
  equip <slot> <index>   装备物品 (slot: head/body/weapon/accessory)
  unequip <slot>         卸下装备
  use <index>            使用物品
  stats                  查看角色属性
  gain_exp <amount>      获得经验 (调试用)
  take_damage <damage>   受到伤害 (调试用)
  heal <amount>          恢复生命值 (调试用)
-----------------------------------------------------------------
  "
                    .into(),
                ));
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

                log.write(LogEvent("--- Equipment ---".into()));
                if let Some(weapon) = &equipment.weapon {
                    log.write(LogEvent(format!(
                        "Weapon: {} (id={})",
                        weapon.proto.name, weapon.proto.id
                    )));
                } else {
                    log.write(LogEvent("Weapon: (empty)".into()));
                }

                log.write(LogEvent("--- Backpack ---".into()));
                let mut empty = true;
                for (i, stack) in backpack.slots.iter().enumerate() {
                    if stack.count > 0 {
                        empty = false;
                        log.write(LogEvent(format!(
                            "[{}] {} ×{} (id={})",
                            i, stack.proto.name, stack.count, stack.proto.id
                        )));
                    }
                }
                if empty {
                    log.write(LogEvent("  (empty)".into()));
                }
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

            Command::Unequip { slot } => {
                ev_unequip.write(crate::equipment::events::UnequipEvent { slot });
            }

            Command::Use { index } => {
                ev_use.write(crate::inventory::events::UseItemEvent { index });
            }

            // Character 相关命令在 execute_character_commands 中处理
            Command::Stats
            | Command::GainExp { .. }
            | Command::TakeDamage { .. }
            | Command::Heal { .. } => {
                // 这些命令由 execute_character_commands 处理
            }

            Command::Unsupported(cmd) => {
                log.write(LogEvent(format!("不支持的命令: {cmd}")));
            }
        }
        
        // 命令处理完毕后重新显示状态栏
        if let Ok(stats) = player_query.single() {
            let status_bar = format!(
                "[HP:{}/{} ATK:{} DEF:{} LV:{}({}/{})] > \n",
                stats.hp, stats.max_hp, stats.atk, stats.def, 
                stats.lv, stats.exp, stats.exp_to_next()
            );
            print!("{}", status_bar);
            io::stdout().flush().unwrap();
        }
    }
}

/// 处理 Character 相关命令
fn execute_character_commands(
    mut line_reader: EventReader<CliLine>,
    _player_query: Query<&Stats, With<Player>>,
    mut ev_show_stats: EventWriter<crate::character::events::ShowStats>,
    mut ev_gain_exp: EventWriter<crate::character::events::GainExp>,
    mut ev_take_damage: EventWriter<crate::character::events::TakeDamage>,
    mut ev_heal: EventWriter<crate::character::events::Heal>,
) {
    for CliLine(input) in line_reader.read() {
        match parse_command(input) {
            Command::Stats => {
                ev_show_stats.write(crate::character::events::ShowStats { entity: None });
            }

            Command::GainExp { amount } => {
                ev_gain_exp.write(crate::character::events::GainExp {
                    entity: Entity::PLACEHOLDER, // 系统会自动查找玩家
                    amount,
                });
            }

            Command::TakeDamage { damage } => {
                ev_take_damage.write(crate::character::events::TakeDamage {
                    entity: Entity::PLACEHOLDER, // 系统会自动查找玩家
                    damage,
                });
            }

            Command::Heal { amount } => {
                ev_heal.write(crate::character::events::Heal {
                    entity: Entity::PLACEHOLDER, // 系统会自动查找玩家
                    amount,
                });
            }

            // 其他命令忽略
            _ => {}
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
        "unequip" => {
            let slot = parts.next().unwrap_or("").to_string();
            Command::Unequip { slot }
        }
        "use" => {
            let idx = parts.next().unwrap_or("0").parse().unwrap_or(0);
            Command::Use { index: idx }
        }
        "stats" => Command::Stats,
        "gain_exp" => {
            let amount = parts.next().unwrap_or("0").parse().unwrap_or(0);
            Command::GainExp { amount }
        }
        "take_damage" => {
            let damage = parts.next().unwrap_or("0").parse().unwrap_or(0);
            Command::TakeDamage { damage }
        }
        "heal" => {
            let amount = parts.next().unwrap_or("0").parse().unwrap_or(0);
            Command::Heal { amount }
        }
        other => Command::Unsupported(other.into()),
    }
}

fn uuid_from_id(id: &str) -> Uuid {
    // 用固定 namespace + id 字节生成版本 5 UUID，保证可重复得到同一值
    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes())
}

/// 显示状态栏系统
fn display_status_bar(
    player_query: Query<&Stats, With<Player>>,
) {
    // 每次都检查是否有玩家存在，如果有则显示状态栏
    if let Ok(stats) = player_query.single() {
        let current_hash = calculate_stats_hash(stats);
        let mut last_hash = LAST_STATS_HASH.lock().unwrap();
        
        // 如果属性发生变化或者是首次显示（hash为0）
        if *last_hash != current_hash {
            *last_hash = current_hash;
            
            // 如果不是首次显示，先清除当前行
            if *last_hash != 0 {
                print!("\r\x1b[K");
            }
            
            let status_bar = format!(
                "[HP:{}/{} ATK:{} DEF:{} LV:{}({}/{})] > ",
                stats.hp, stats.max_hp, stats.atk, stats.def, 
                stats.lv, stats.exp, stats.exp_to_next()
            );
            
            print!("{}", status_bar);
            io::stdout().flush().unwrap();
        }
    }
}

/// 计算属性哈希值用于检测变化
fn calculate_stats_hash(stats: &Stats) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    stats.hp.hash(&mut hasher);
    stats.max_hp.hash(&mut hasher);
    stats.atk.hash(&mut hasher);
    stats.def.hash(&mut hasher);
    stats.lv.hash(&mut hasher);
    stats.exp.hash(&mut hasher);
    hasher.finish()
}
