use bevy::prelude::*;

/// 装备变更事件 - 触发属性重新计算
#[derive(Event)]
pub struct EquipmentChanged {
    pub entity: Entity,
}

/// 获得经验事件
#[derive(Event)]
pub struct GainExp {
    pub entity: Entity,
    pub amount: i32,
}

/// 受到伤害事件
#[derive(Event)]
pub struct TakeDamage {
    pub entity: Entity,
    pub damage: i32,
}

/// 死亡事件
#[derive(Event)]
pub struct Death {
    pub entity: Entity,
}

/// 升级事件
#[derive(Event)]
pub struct LevelUp {
    pub entity: Entity,
    pub new_level: i32,
}

/// 治疗事件
#[derive(Event)]
pub struct Heal {
    pub entity: Entity,
    pub amount: i32,
}

/// 属性重新计算事件
#[derive(Event)]
pub struct RecalculateStats {
    pub entity: Entity,
}

/// 显示属性事件（用于命令行）
#[derive(Event)]
pub struct ShowStats {
    pub entity: Option<Entity>,  // None 表示显示玩家属性
}
