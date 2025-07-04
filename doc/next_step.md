完成每个里程碑时，附带一段 `cargo run` → 输入命令序列示例，确保逻辑正确并可持续集成。
# 下一步功能开发计划（CLI ASCII 地图·步进式实时制 · 详细版）

本计划基于 **命令行‑ASCII** 原型，旨在按“玩家/AI 行动 ➜ 世界推进一格”的 **步进式实时** 思路，逐层筑起可持续扩展的 roguelike 架构。每阶段都产出一个 _可运行 + 可测试_ 的最小版本，并在 CI 中加入自动化脚本验证。

> **约定**
> - 坐标系：`(x, y)`，左上为 `(0, 0)`，x→右，y→下
> - 时间片：1 Tick = 100 ms（默认），可在无渲染环境调至 50 ms
> - 命名：资源/组件 `PascalCase`，系统/函数 `snake_case`

---

## 阶段 1：角色属性与背包整合（基础数值层）
> _从「纯数据」层面让角色“活”起来_
要求: Stats能通过命令更新, 也会在每次任何命令后刷新, 也就类似状态条是常态化显示的, 一般情况不需要命令更新,因为任何命令都会更新他,添加命令只是冗余设计
### 1.1 Stats 组件
| 字段 | 说明 | 默认 |
|------|------|------|
| `hp / max_hp` | 当前/最大生命 | 20 |
| `atk / def` | 攻防力 | 2 / 1 |
| `lv / exp`  | 等级与经验 | 1 / 0 |
| `rng` | 基础攻击距离 | 1 |

- **升级公式**：`exp_to_next = 10 × lv²`
  每升一级：`+2 max_hp`, `+1 atk`, `+1 def`

### 1.2 事件流
```
EquipmentChanged → RecalculateStats
GainExp(xp)      → TryLevelUp
TakeDamage(dmg)  → If hp<=0 ⇒ Death
```

### 1.3 背包/装备
- `Inventory { slots: Vec<Option<Item>>, capacity: 20 }`
- `Equipment { head, body, weapon, accessory }`
- `item_use` 系统：根据 `ItemKind`
  - `Potion`: 恢复 HP
  - `Scroll`: 单次 Buff
  - `Key{id}`: 与门匹配
- `apply_equipment_bonuses`：扫描 `Equipment` → 累加到 Stats

### 1.4 测试矩阵
| 场景 | 步骤 | 预期 |
|------|------|------|
| 装备‑卸载 | equip → unequip | atk/def 增减一致 |
| 升级 | gain exp 100 | lv ↑，max_hp+2 |


---

## 阶段 2：地图与渲染核心（可视化层）
> _把抽象网格投射成 CLI 可读地图_

### 2.1 TileMap 资源
```rust
pub struct TileMap {
    pub w: u32,
    pub h: u32,
    pub tiles: Vec<Tile>,                // 长度 = w*h
    pub occupied: HashMap<Coord, Entity> // 实体占用
}
```
- **API**
  - `fn idx(&self, x, y) -> usize`
  - `fn is_walkable(&self, coord) -> bool`
  - `fn load_from_csv(path)` / `from_template(&str)`

### 2.2 碰撞规则
1. `Wall` 不可进入
2. `Door` 可进入，若未解锁则视为 `Wall`
3. 若目标格被敌对 `Actor` 占据 → 触发战斗

### 2.3 ASCII Renderer
- 清屏 → 打印 `w+2` 列边框，上下留一行 HUD (`HP`, `Lv`, 坐标)
- 渲染顺序：背景 Tile → 遮挡 (FOV) → 实体层
- **优化**：
  - 记录上一帧字符串，若相同则跳过打印
  - 可切换 “全刷新” / “差异刷新”

### 2.4 命令
| 输入 | 行为 |
|------|------|
| `look` | 仅渲染地图（无行动） |
| `map`  | 打印完整未遮挡地图（调试） |

### 2.5 单元/集成测试
- `TileMap` 解析 CSV → 等效 Vec 长度
- `render_map` 对固定输入输出断言（snapshot test）

---

## 阶段 3：Actor 与调度（时间逻辑层）
> _让世界“动”起来_

### 3.1 Actor/Coord 组件
```rust
struct Actor { speed: u8, faction: Faction }
enum Faction { Player, Monster, Neutral }
struct Coord { x: i32, y: i32 }
```
- `speed` → 每 Tick 增 `energy += speed`; 当 `energy >= 100` 则行动一次并扣除 100

### 3.2 TurnScheduler 资源
- `BinaryHeap<(Reverse<energy>, Entity)>`
- 系统 `refill_queue`：遍历 Actor → push if `energy >= 100`
- 系统 `process_turn`：pop → 发送 `TakeTurn` 事件

### 3.3 AI 行为
- **PathCache**：仅在玩家移动或地图改变时重新执行 A*，避免每 Tick 计算
- **Wander**：随机 4 方向，如果阻塞则原地等待（`Wait` 动作消耗 50 energy）

### 3.4 调试
- 命令 `ai on/off` 打印怪物意图
- 系统 `log_turns`：在 `--debug` 模式输出每个行动者的描述

---

## 阶段 4：战斗与互动（核心玩法层）
> _让“移动 + 交互 + 伤害”闭环_

### 4.1 伤害流水
```
AttackIntent { from, to } → damage_system
damage_system:
    hit_roll = rng(1..100) <= 90  // 10% miss
    dmg = max(1, atk - def) ± rng(-1..1)
    emit TakeDamage(to, dmg)
```
- **暴击**：5% 概率  ×2 伤害
- **远程**：若 `rng > 1` → 创建 `Projectile` 实体，按 Tick 移动

### 4.2 死亡与掉落
- `Death` 事件 →
  1. 生成 `corpse` Tile（字符 `%`）
  2. `LootTable` → 随机 `Item` 放置于同格
  3. 若目标是 Monster → 撒 `ExpOrb`（自动拾取）

### 4.3 互动命令
| 命令 | 说明 |
|-----|-----|
| `open` | 若前方是 `Door` 或 `Chest` → 尝试打开 |
| `talk` | 若前方是 NPC → 触发对话脚本 |
| `examine` | 输出目标格详细信息 |

### 4.4 测试
- 100 次模拟战斗确保 **平均 DPS** ≥ 理论值 ×0.9
- `pickup→use Potion` 回血量与日志一致

---

## 阶段 5：地图生成与事件（内容层）
> _带来“每次不同”的可重复乐趣_

### 5.1 楼层生成器
1. BSP 切割 → 递归到 `min_room=4×4`
2. 随机连接房间 → 确保单连通
3. 装饰：
   - `MonsterSpawner` = `floor_tiles × 5%`
   - `Treasure` = `rooms × 25%`
   - 随机锁住一扇门并放 `Key` 最远区
4. 保存 RNG `seed` 以便复现

### 5.2 事件系统
- 统一 `MapEvent`：
  - `OnEnter{coord, entity}`
  - `OnUse{coord, entity}`
- 具体事件实现为 **标签组件** (`Trap`, `Fountain`, `Portal`)
- `trigger_events` 系统：根据标签派发脚本

### 5.3 多楼层
- `world_level: i32` 资源
- `StairUp` / `StairDown` Tile：切换 TileMap 栈，保留 Player Stats

### 5.4 测试
- 100 次生成：保证 `出口` 可达率 100%
- 生成‑加载‑保存‑复现同一 seed，地图一致

---

## 目录结构（增量）
```
src/
├── character/
│   ├── components.rs    # Stats / Inventory / Equipment
│   └── systems.rs
├── world/
│   ├── tile.rs
│   ├── map.rs
│   └── generation.rs    # BSP + decorators
├── render/
│   └── ascii.rs
├── actor/
│   ├── components.rs
│   ├── scheduler.rs
│   ├── ai.rs
│   └── actions.rs       # Move / Wait / Attack intent structs
├── combat/
│   ├── damage.rs
│   └── systems.rs
└── events.rs            # Centralised event enums
```

---

## 实现里程碑（更新）
| 里程碑 | 所含阶段 | 交付物 | 自动化测试 |
|-------|---------|-------|-----------|
| **M1** | Phase 1 | 角色能穿戴装备 & 升级 | stats / level‑up 单测 |
| **M2** | Phase 2 | `look` 渲染地图 & HUD | TileMap 解析 + 渲染快照 |
| **M3** | Phase 3 | AI 与玩家交替行动 | 行动序列一致性 |
| **M4** | Phase 4 | 战斗闭环、掉落、拾取 | 100 场模拟战 |
| **M5** | Phase 5 | 随机楼层 + 事件格 | 生成可达性 & 种子复现 |

每到里程碑，附示范脚本：
```
cargo run -- --seed 123
> look
> w a a pickup
> d d attack
...
```

> 完成后可继续迭代：
> - 日志系统 / 重播
> - JSON‑RPC 远程控制 / 多人回放
> - 渲染器替换为 `bevy_ascii_terminal` 获取彩色输出