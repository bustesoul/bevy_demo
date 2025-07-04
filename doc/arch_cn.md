# 项目架构文档

## 1. 概述

本文档详细介绍了当前项目的软件架构、核心逻辑、模块划分以及业务流程。
本项目使用 Rust 语言和 Bevy 游戏引擎构建，旨在创建一个可扩展的、基于文本/控制台的游戏框架。
当前实现的核心是一个无 UI 的 Bevy 应用，通过命令行进行交互。

## 2. 整体架构

项目采用模块化的设计，遵循 Bevy 引擎的实体-组件-系统 (ECS) 模式。主要分为以下几个核心模块：

-   `core`: 核心插件，负责定义和管理全局状态、事件和资源。
-   `character`: 角色系统，管理角色属性、升级和事件处理。
-   `data`: 数据管理模块，负责从外部文件（如 RON）加载游戏数据（如物品、怪物等）。
-   `inventory`: 背包模块，管理玩家物品和使用逻辑。
-   `equipment`: 装备模块，管理角色四个槽位的装备。
-   `interface`: 用户交互模块，目前实现了一个简单的调试命令行界面 (CLI)。
-   `main.rs`: 应用入口，负责组装所有插件和系统，启动 Bevy 应用。

### 2.1. 技术栈

-   **语言**: Rust
-   **游戏引擎**: Bevy
-   **数据格式**: RON (Rusty Object Notation)

### 2.2. 启动流程

1.  `main.rs` 中的 `main` 函数启动 Bevy `App`。
2.  加载 `DefaultPlugins`，但禁用了主窗口的显示 (`visible: false`)，实现“无头”运行模式。
3.  加载自定义的 `CorePlugin`, `CharacterPlugin`, `DataPlugin`, `InventoryPlugin`, `EquipmentPlugin`, `DebugCliPlugin`，初始化应用的核心功能。
4.  `main.rs` 中的 `Startup` 系统将 `AppState` 切换到 `Loading`。
5.  `DataPlugin` 中的系统负责在 `Loading` 状态加载数据，加载完成后将 `AppState` 切换到 `InGame`。
6.  `DebugCliPlugin` 启动一个独立线程，通过标准输入(stdin)与用户进行命令行交互。

## 3. 模块详解

### 3.1. `main.rs`

作为应用的入口点，`main.rs` 的职责是：
-   初始化 Bevy `App`。
-   配置 `DefaultPlugins`，特别是 `WindowPlugin`，以隐藏图形界面。
-   注册项目的所有自定义插件：`CorePlugin`, `character::CharacterPlugin`, `data::DataPlugin`, `inventory::InventoryPlugin`, `equipment::EquipmentPlugin`, `interface::DebugCliPlugin`。
-   注册一个全局的日志转发系统 `forward_log_event`，它会监听 `core::events::LogEvent` 事件并将其内容打印到控制台。
-   注册一个 `Startup` 系统，在应用启动时将 `AppState` 设置为 `Loading`，从而启动数据加载流程。

### 3.2. `core` 模块

`core` 模块是整个应用的核心，定义了最基本的数据结构和规则。

-   **`CorePlugin` (`src/core/mod.rs`)**:
    -   这是 `core` 模块的入口插件。
    -   在 `build` 方法中，它向 Bevy `App` 注册了：
        -   `AppState`: 全局应用状态机。
        -   `LogEvent`: 一个简单的日志事件，用于在不同系统间传递日志信息。
        -   `GameConfig`: 一个全局资源，用于存储游戏配置。
        -   `hello_world` system: 一个在 `Startup` 阶段运行的示例系统，用于演示事件发送。

-   **`states.rs` (`src/core/states.rs`)**:
    -   定义了 `AppState` 枚举，管理应用的宏观状态。
    -   `Startup`: 应用启动时的初始状态。
    -   `Loading`: 加载游戏资源的状态。
    -   `InGame`: 游戏主循环，可以进行交互的状态。
    -   `Shutdown`: 准备关闭应用的状态。

-   **`events.rs` (`src/core/events.rs`)**:
    -   定义了 `LogEvent(String)`，一个元组结构体事件，用于承载日志消息。
    -   `hello_world` 系统演示了如何发送事件：它在启动时发送一个 "Hello, Bevy!" 的 `LogEvent`。

-   **`resources.rs` (`src/core/resources.rs`)**:
    -   定义了 `GameConfig` 资源，包含游戏配置项。资源是 Bevy 中存储全局单例数据的方式。

### 3.3. `character` 模块

`character` 模块是游戏的核心业务逻辑之一，负责管理角色的所有属性、状态和行为。

-   **`CharacterPlugin` (`src/character/mod.rs`)**:
    -   注册 `Stats` 组件，并将其添加到一个代表玩家的实体上。
    -   注册 `GainExpEvent`, `TakeDamageEvent`, `HealEvent`, `RecalculateStatsEvent`, `LevelUpEvent` 等事件。
    -   注册 `gain_exp`, `take_damage`, `heal`, `recalculate_stats`, `try_level_up`, `check_death` 等系统。

-   **`components.rs` (`src/character/components.rs`)**:
    -   `Stats`: 一个组件，包含了角色的所有核心属性，如 `hp`/`max_hp`, `atk`/`def`, `lv`/`exp` 等。
    -   它还包含了计算逻辑，如 `exp_to_next()` (升级所需经验) 和 `level_up()` (执行升级)。

-   **`events.rs` (`src/character/events.rs`)**:
    -   定义了与角色状态变化相关的各种事件，如 `GainExpEvent` (获得经验), `TakeDamageEvent` (受到伤害) 等。

-   **`systems.rs` (`src/character/systems.rs`)**:
    -   `recalculate_stats`: 监听 `RecalculateStatsEvent` (通常由装备变更触发)，重新计算角色的最终属性（基础属性 + 装备加成）。
    -   `gain_exp` & `try_level_up`: 监听 `GainExpEvent`，增加经验值，并检查是否满足升级条件，如果满足则触发升级逻辑。
    -   `take_damage` & `check_death`: 监听 `TakeDamageEvent`，处理伤害计算和死亡判断。

### 3.4. `data` 模块

`data` 模块负责游戏数据的加载和管理。

-   **`DataPlugin` (`src/data/mod.rs`)**:
    -   注册 `ItemList` 资产类型和 `RonItemLoader`。
    -   注册 `ItemAssets` 资源，用于存储加载后资产的句柄。
    -   注册 `OnEnter(AppState::Loading)` 状态的 `start_loading` 系统，用于启动资产加载。
    -   注册 `Update` 状态的 `check_loaded` 系统（在 `Loading` 状态下运行），用于检查加载进度并切换到 `InGame` 状态。

-   **`schema.rs` (`src/data/schema.rs`)**:
    -   定义了游戏数据的结构。
    -   `ItemEntry`: 代表单个物品的结构，包含 `id`, `name`, `atk`, `heal` 等属性。使用了 `serde` 来支持反序列化。
    -   `ItemList`: 代表一个物品列表，是 `RonItemLoader` 加载的目标资源类型。

-   **`loader.rs` (`src/data/loader.rs`)**:
    -   实现了 `RonItemLoader`，这是一个自定义的 `AssetLoader`。
    -   它负责从 `.ron` 文件中异步读取物品数据，并将其解析为 `ItemList` 类型的资产。
    -   定义了 `RonItemLoaderError` 来处理 IO 和解析过程中可能出现的错误。

-   **`assets/data/items.ron`**:
    -   一个数据文件示例，使用 RON 格式定义了一个物品列表。

### 3.5. `interface` 模块

`interface` 模块是用户与游戏交互的桥梁。

-   **`DebugCliPlugin` (`src/interface/debug_cli.rs`)**:
    -   插件在 `build` 时，会启动一个新线程专门用于阻塞式地读取标准输入（`stdin`）。
    -   读取到的每一行输入都会被存入一个全局静态的线程安全队列 `CLI_BUFFER`。
    -   `read_stdin` 系统：在 Bevy 的主线程中，每帧检查 `CLI_BUFFER`，取出所有行并作为 `CliLine` 事件发送。
    -   `execute_cli_commands` 系统：此系统仅在 `AppState::InGame` 状态下运行。它监听 `CliLine` 事件，解析命令并执行相应操作。它会查询 `Res<T>` 来获取世界信息，并通过发送事件（如 `GiveItemEvent`, `EquipEvent`, `UnequipEvent`, `UseItemEvent`, `GainExpEvent`, `TakeDamageEvent`, `LogEvent` 等）来驱动游戏逻辑或输出信息。
    -   `parse_command` 函数：将用户输入的字符串（如 "give sword_iron 1"）解析为结构化的 `Command` 枚举（如 `Command::Give { id: "sword_iron", count: 1 }`）。
    -   支持的命令包括 `help`, `status`, `stats`, `exit`, `items`, `give`, `inventory`, `equip`, `unequip`, `use`, `gain_exp`, `take_damage`, `heal` 等。
    -   `uuid_from_id` 工具函数：根据物品的字符串 `id` 生成一个稳定的版本5 UUID，确保其唯一性和可重复性。

### 3.6. `inventory` 模块

`inventory` 模块负责管理玩家的背包。

-   **`InventoryPlugin` (`src/inventory/mod.rs`)**:
    -   注册 `Backpack` 资源，作为玩家的背包实例，并设定初始容量。
    -   注册 `GiveItemEvent`, `ListInventoryEvent`, `UseItemEvent` 事件。
    -   注册 `give_item`, `print_inventory`, `use_item` 系统，它们在 `InGame` 状态下运行。

-   **`components.rs` (`src/inventory/components.rs`)**:
    -   `Backpack`: 一个资源，包含一个 `Vec<ItemStack>` (slots) 和容量 (capacity)。
    -   `ItemStack`: 代表一叠物品，包含物品原型 (`ItemEntry`) 和数量 (`count`)。

-   **`events.rs` (`src/inventory/events.rs`)**:
    -   `GiveItemEvent`: 请求向背包添加物品的事件。
    -   `ListInventoryEvent`: 请求打印背包内容的事件。
    -   `UseItemEvent`: 请求使用背包中物品的事件。

-   **`systems.rs` (`src/inventory/systems.rs`)**:
    -   `give_item`: 监听 `GiveItemEvent`，根据物品 ID 查找原型，并将其添加到 `Backpack` 资源中。它会处理物品堆叠和寻找空位。
    -   `print_inventory`: 监听 `ListInventoryEvent`，遍历 `Backpack` 并将内容打印到标准输出。
    -   `use_item`: 监听 `UseItemEvent`，处理使用物品的逻辑，如恢复生命值（药水）、触发临时效果（卷轴）等。

### 3.7. `equipment` 模块

`equipment` 模块负责管理玩家的装备。

-   **`EquipmentPlugin` (`src/equipment/mod.rs`)**:
    -   注册 `Equipment` 资源。
    -   注册 `EquipEvent` 和 `UnequipEvent` 事件。
    -   注册 `equip_item` 和 `unequip_item` 系统，在 `InGame` 状态下运行。

-   **`components.rs` (`src/equipment/components.rs`)**:
    -   `Equipment`: 一个资源，包含四个装备槽位：`head`, `body`, `weapon`, `accessory`，均为 `Option<ItemStack>`。

-   **`events.rs` (`src/equipment/events.rs`)**:
    -   `EquipEvent`: 请求装备物品的事件，包含槽位名称和背包索引。
    -   `UnequipEvent`: 请求卸下装备的事件，包含槽位名称。

-   **`systems.rs` (`src/equipment/systems.rs`)**:
    -   `equip_item`: 监听 `EquipEvent`，处理从背包中取出物品并装备到相应槽位的逻辑。如果槽位已有装备，则将其放回背包。
    -   `unequip_item`: 监听 `UnequipEvent`，处理将装备从槽位卸下并放回背包的逻辑。

## 4. 逻辑流程

### 4.1. 应用启动与数据加载

1.  **启动**: `main` 函数执行，创建 Bevy `App` 并注册所有插件和系统。
2.  **进入 Loading**: `main.rs` 中的 `Startup` 系统立即将 `AppState` 从 `Default` (`Startup`) 切换到 `Loading`。
3.  **开始加载**:
    -   进入 `Loading` 状态后，`data` 模块的 `start_loading` 系统被触发。
    -   它通过 `AssetServer` 请求加载 `assets/data/items.ron` 文件，并将返回的 `Handle` 存入 `ItemAssets` 资源。
    -   Bevy 的 `AssetServer` 会调用 `RonItemLoader` 来处理 `.ron` 文件。
4.  **检查加载状态**:
    -   在 `Loading` 状态的每一帧，`data` 模块的 `check_loaded` 系统都会运行。
    -   它检查 `ItemAssets` 中的句柄是否已经加载完毕。
5.  **进入游戏**: 资源加载完成后，`check_loaded` 系统将 `AppState` 切换到 `InGame`。

### 4.2. 游戏内交互 (CLI)

1.  **等待输入**: 应用进入 `InGame` 状态后，`execute_cli_commands` 系统开始运行，等待 `CliLine` 事件。同时，`stdin` 读取线程持续监听用户输入。
2.  **读取与事件化**: 用户在控制台输入命令并按回车后，`stdin` 线程读取该行，并将其放入 `CLI_BUFFER`。`read_stdin` 系统在下一帧发现并将其包装成 `CliLine` 事件。
3.  **命令解析与执行**:
    -   `execute_cli_commands` 系统接收到 `CliLine` 事件。
    -   调用 `parse_command` 将字符串解析为 `Command` 枚举。
    -   根据 `Command` 的类型，发送相应的事件来驱动游戏逻辑：
        -   `Command::Give` → `GiveItemEvent`
        -   `Command::Inventory` → `ListInventoryEvent`
        -   `Command::Equip` → `EquipEvent`
        -   `Command::Unequip` → `UnequipEvent`
        -   `Command::Use` → `UseItemEvent`
        -   `Command::GainExp` → `GainExpEvent`
        -   `Command::TakeDamage` → `TakeDamageEvent`
        -   `Command::Exit` → `AppExit`
        -   对于查询类命令 (`help`, `status`, `stats`, `items`)，系统会直接查询数据并发送 `LogEvent` 来显示信息。
4.  **响应输出**: `main.rs` 中的 `forward_log_event` 系统捕获所有 `LogEvent`，并将其内容用 `println!` 打印到控制台，从而向用户显示结果。

### 4.3. 核心事件驱动流程

项目中的许多核心功能是通过事件链驱动的，实现了模块间的解耦。

1.  **装备变更与属性重算**:
    -   用户执行 `equip` 或 `unequip` 命令。
    -   `equipment` 模块的 `equip_item` / `unequip_item` 系统处理装备逻辑。
    -   成功后，发送 `EquipmentChanged` 事件。
    -   `character` 模块的 `schedule_recalculate_stats` 系统监听到 `EquipmentChanged`，发送 `RecalculateStatsEvent`。
    -   `recalculate_stats` 系统执行，根据玩家当前的基础属性和所有已装备物品的加成，更新最终的 `Stats`。

2.  **获得经验与升级**:
    -   用户执行 `gain_exp` 命令（或将来通过战斗获得经验）。
    -   `character` 模块的 `gain_exp` 系统监听到 `GainExpEvent`，增加 `Stats` 中的 `exp`。
    -   该系统接着调用 `Stats::can_level_up()` 检查是否可以升级。
    -   如果可以，则发送 `LevelUpEvent`。
    -   `try_level_up` 系统监听到 `LevelUpEvent`，调用 `Stats::level_up()` 执行升级，并发送 `LogEvent` 通知玩家。

3.  **使用物品**:
    -   用户执行 `use` 命令。
    -   `inventory` 模块的 `use_item` 系统监听到 `UseItemEvent`。
    -   系统检查物品类型（如 `Potion`），并发送相应的效果事件（如 `HealEvent`）。
    -   `character` 模块的 `heal` 系统监听到 `HealEvent`，恢复玩家的 `hp`。

## 5. 总结

当前项目是一个结构清晰、模块化的 Bevy 应用框架。它成功地将核心逻辑 (`core`)、数据处理 (`data`)、业务功能（`character`, `inventory`, `equipment`）和用户界面 (`interface`) 分离开来。通过利用 Bevy 的状态机和事件系统，实现了清晰的业务流程控制和功能解耦。基于这个框架，可以方便地扩展新的游戏功能，例如添加战斗系统、地图生成、任务系统等。
