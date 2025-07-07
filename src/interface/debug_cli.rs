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

static PENDING_OUTPUTS: Lazy<Arc<Mutex<VecDeque<String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

static GAME_LOG_HISTORY: Lazy<Arc<Mutex<VecDeque<GameLogEntry>>>> =
    Lazy::new(|| Arc::new(Mutex::new(VecDeque::new())));

static CURRENT_GAME_ENTRY: Lazy<Arc<Mutex<Option<GameLogEntry>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

static UI_STATE: Lazy<Arc<Mutex<UIState>>> = 
    Lazy::new(|| Arc::new(Mutex::new(UIState::default())));

#[derive(Default)]
struct UIState {
    last_stats_hash: u64,
    show_status_bar: bool,
    needs_refresh: bool,
    current_input: String,
    cursor_position: usize,
}

/// 游戏日志条目
#[derive(Clone)]
struct GameLogEntry {
    input: Option<String>,  // 用户输入的命令
    outputs: Vec<String>,   // 对应的输出内容
}

/// 命令类型分类
#[derive(Debug, Clone, PartialEq)]
enum CommandType {
    System,    // 系统命令：help, status, items, inventory等
    Game,      // 游戏命令：gain_exp, take_damage, heal, equip, use等
}

/// CLI消息类型
#[derive(Clone)]
enum CliMessage {
    UserInput(String, CommandType),    // 用户输入的命令及其类型
    SystemResponse(String),            // 系统命令的即时响应
    GameLog(String),                  // 游戏日志条目
    Info(String),
    Success(String),
    Warning(String),
    Error(String),
}

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
            // log_cli_input: 在读取输入后立即记录命令
            .add_systems(Update, log_cli_input.run_if(in_state(AppState::InGame)).after(read_stdin))
            // UI渲染系统
            .add_systems(Update, render_ui.run_if(in_state(AppState::InGame)).after(log_cli_input))
            // 命令执行，确保在日志和 UI 之后（即实际执行逻辑与打印解耦）
            .add_systems(
                Update,
                (
                    execute_basic_commands,
                    execute_character_commands,
                )
                    .run_if(in_state(AppState::InGame))
                    .after(render_ui),
            )
            // 游戏开始时初始化UI
            .add_systems(OnEnter(AppState::InGame), initialize_ui);
/// 日志记录：在读取到每一行命令时，立即入队 UserInput（保证顺序）
fn log_cli_input(mut line_reader: EventReader<CliLine>) {
    for CliLine(input) in line_reader.read() {
        let command = parse_command(&input);
        let ty = command.command_type();
        queue_output(CliMessage::UserInput(input.clone(), ty));
    }
}
    }
}

/* ---------------------------- 事件与枚举 ---------------------------- */

/// 终端敲的一整行
#[derive(Event)]
struct CliLine(String);

/// 我们支持的命令
enum Command {
    // 系统命令
    Help,
    Status,
    Exit,
    Items(Option<String>), // None=全部；Some(token)=按 id/uuid/name 查询
    Inventory,
    Stats,

    // 游戏命令
    Give { id: String, count: u32 },
    Equip { slot: String, index: usize },
    Unequip { slot: String },
    Use { index: usize },
    GainExp { amount: i32 },
    TakeDamage { damage: i32 },
    Heal { amount: i32 },

    Unsupported(String),
}

impl Command {
    /// 获取命令类型
    fn command_type(&self) -> CommandType {
        match self {
            Command::Help | Command::Status | Command::Exit |
            Command::Items(_) | Command::Inventory | Command::Stats => CommandType::System,

            Command::Give { .. } | Command::Equip { .. } | Command::Unequip { .. } |
            Command::Use { .. } | Command::GainExp { .. } | Command::TakeDamage { .. } |
            Command::Heal { .. } => CommandType::Game,

            Command::Unsupported(_) => CommandType::System,
        }
    }
}

/* ---------------------------- 读取 stdin ---------------------------- */

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
    _player_query: Query<&Stats, With<Player>>,
    mut ev_give: EventWriter<crate::inventory::events::GiveItemEvent>,
    mut ev_list: EventWriter<crate::inventory::events::ListInventoryEvent>,
    mut ev_equip: EventWriter<crate::equipment::events::EquipEvent>,
    mut ev_unequip: EventWriter<crate::equipment::events::UnequipEvent>,
    mut ev_use: EventWriter<crate::inventory::events::UseItemEvent>,
) {
    for CliLine(input) in line_reader.read() {
        let command = parse_command(input);
        // let command_type = command.command_type();
        // queue_output(CliMessage::UserInput(input.clone(), command_type.clone()));
        match command {
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

/* ---------------------------- UI 系统 ---------------------------- */

/// 队列输出消息
fn queue_output(message: CliMessage) {
    let mut outputs = PENDING_OUTPUTS.lock().unwrap();
    let mut game_history = GAME_LOG_HISTORY.lock().unwrap();
    let mut current_entry = CURRENT_GAME_ENTRY.lock().unwrap();
    
    match message {
        CliMessage::UserInput(cmd, command_type) => {
            match command_type {
                CommandType::System => {
                    // 系统命令：即时显示，不加入历史
                    outputs.push_back(format!("┌─ 系统命令 ─{}", "─".repeat(50)));
                    outputs.push_back(format!("│ > {}", cmd));
                    outputs.push_back(format!("└{}", "─".repeat(58)));
                }
                CommandType::Game => {
                    // 完成当前游戏日志条目（如果有的话）
                    if let Some(entry) = current_entry.take() {
                        game_history.push_back(entry);
                    }
                    
                    // 开始新的游戏日志条目
                    *current_entry = Some(GameLogEntry {
                        input: Some(cmd.clone()),
                        outputs: Vec::new(),
                    });
                    
                    // 即时显示
                    outputs.push_back(format!("┌─ 游戏命令 ─{}", "─".repeat(50)));
                    outputs.push_back(format!("│ > {}", cmd));
                    outputs.push_back(format!("└{}", "─".repeat(58)));
                }
            }
        }
        CliMessage::SystemResponse(msg) => {
            // 系统响应：即时显示，不加入历史
            outputs.push_back(format!("📋 {}", msg));
        }
        CliMessage::GameLog(msg) => {
            // 游戏日志：添加到当前游戏条目的输出中
            let formatted_msg = format!("📝 {}", msg);
            outputs.push_back(formatted_msg.clone());
            
            if let Some(ref mut entry) = current_entry.as_mut() {
                entry.outputs.push(formatted_msg);
            } else {
                // 如果没有当前条目，创建一个只有输出的条目
                game_history.push_back(GameLogEntry {
                    input: None,
                    outputs: vec![formatted_msg],
                });
            }
        }
        CliMessage::Info(msg) => {
            let line = format!("  ℹ️  {}", msg);
            outputs.push_back(line);
            // Info消息不自动加入游戏历史，只有明确的GameLog才加入
        }
        CliMessage::Success(msg) => {
            let line = format!("  ✅ {}", msg);
            outputs.push_back(line);
            // Success消息不自动加入游戏历史
        }
        CliMessage::Warning(msg) => {
            let line = format!("  ⚠️  {}", msg);
            outputs.push_back(line);
            // Warning消息不自动加入游戏历史
        }
        CliMessage::Error(msg) => {
            let line = format!("  ❌ {}", msg);
            outputs.push_back(line);
            // Error消息不自动加入游戏历史
        }
    }
    
    // 保持游戏历史记录最多10条条目
    const MAX_GAME_ENTRIES: usize = 10;
    while game_history.len() > MAX_GAME_ENTRIES {
        game_history.pop_front();
    }
}

/// 公开函数：供main.rs调用，将LogEvent转换为格式化消息
pub fn queue_log_message(message: String) {
    queue_output(CliMessage::Info(message));
}

/// 公开函数：添加游戏日志条目
pub fn queue_game_log(message: String) {
    queue_output(CliMessage::GameLog(message));
}

/// 主UI渲染系统
fn render_ui(player_query: Query<&Stats, With<Player>>) {
    let mut ui_state = UI_STATE.lock().unwrap();
    let mut outputs = PENDING_OUTPUTS.lock().unwrap();
    let game_history = GAME_LOG_HISTORY.lock().unwrap();
    let current_entry = CURRENT_GAME_ENTRY.lock().unwrap();
    
    // 检查是否有待输出的消息
    let has_outputs = !outputs.is_empty();
    
    // 检查玩家属性是否变化
    let mut stats_changed = false;
    if let Ok(stats) = player_query.single() {
        let current_hash = calculate_stats_hash(stats);
        if ui_state.last_stats_hash != current_hash {
            ui_state.last_stats_hash = current_hash;
            stats_changed = true;
            ui_state.show_status_bar = true;
        }
    }
    
    // 如果有任何更新，重新渲染整个界面
    if has_outputs || stats_changed || ui_state.needs_refresh {
        // 清屏（全屏刷新）
        print!("\x1b[2J\x1b[H");
        // 对于系统命令，只显示即时输出
        if has_outputs {
            // 输出所有待处理的消息
            while let Some(output) = outputs.pop_front() {
                println!("{}", output);
            }
        }
        
        // 如果有游戏日志历史或需要刷新，显示完整界面
        if !game_history.is_empty() || current_entry.is_some() || ui_state.needs_refresh {
            if has_outputs {
                println!("{}", "═".repeat(80));
            }
            
            // 显示游戏日志历史（从旧到新的顺序）
            if !game_history.is_empty() || current_entry.is_some() {
                println!("📊 游戏日志 (最新记录在下方):");
                println!("{}", "─".repeat(80));

                // 显示历史记录
                for entry in game_history.iter() {
                    display_game_log_entry(entry);
                }

                // 显示当前正在进行的条目
                if let Some(ref entry) = current_entry.as_ref() {
                    display_game_log_entry(entry);
                }

                println!("{}", "═".repeat(80));
            }
            
            // 显示状态栏
            if let Ok(stats) = player_query.single() {
                let status_bar = format_status_bar(stats);
                println!("{}", status_bar);
            }
            
            // 显示输入提示区域
            display_input_area();
        } else if has_outputs {
            // 仅有系统命令输出时，简单显示输入提示
            println!("{}", "─".repeat(80));
            print!("》 ");
            io::stdout().flush().unwrap();
        }
        
        ui_state.needs_refresh = false;
    }
}

/// 显示单个游戏日志条目
fn display_game_log_entry(entry: &GameLogEntry) {
    // 先显示用户输入
    if let Some(ref input) = entry.input {
        println!("🎮 > {}", input);
    }

    // 然后显示对应的输出（从上到下）
    for output in &entry.outputs {
        println!("  {}", output);
    }
}

/// 显示输入区域
fn display_input_area() {
    println!("{}", "─".repeat(80));
    print!("》 ");
    io::stdout().flush().unwrap();
}

/// 格式化状态栏
fn format_status_bar(stats: &Stats) -> String {
    format!(
        "📊 [HP:{}/{} ⚔️ATK:{} 🛡️DEF:{} 📈LV:{}({}/{})]",
        stats.hp, stats.max_hp, stats.atk, stats.def, 
        stats.lv, stats.exp, stats.exp_to_next()
    )
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

/// 初始化UI界面
fn initialize_ui() {
    // 清屏
    print!("\x1b[2J\x1b[H");
    
    // 显示欢迎界面
    println!("{}", "═".repeat(80));
    println!("🎮 欢迎来到文字RPG游戏！");
    println!("{}", "═".repeat(80));
    println!("📖 游戏说明:");
    println!("   • 这是一个基于Bevy引擎的文字RPG游戏");
    println!("   • 你可以通过命令与游戏交互");
    println!("   • 输入 'help' 查看所有可用命令");
    println!("   • 你的角色属性会实时显示在状态栏中");
    println!("{}", "═".repeat(80));
    println!("{}", "─".repeat(80));
    print!("》 ");
    io::stdout().flush().unwrap();
    
    // 标记UI需要刷新
    let mut ui_state = UI_STATE.lock().unwrap();
    ui_state.needs_refresh = true;
    ui_state.show_status_bar = true;
}
