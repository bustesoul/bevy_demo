use super::{components::*, events::*};
use crate::core::events::LogEvent;
use crate::equipment::components::Equipment;
use bevy::prelude::*;

/// 处理获得经验事件
pub fn handle_gain_exp(
    mut ev_gain_exp: EventReader<GainExp>,
    mut ev_level_up: EventWriter<LevelUp>,
    mut ev_log: EventWriter<LogEvent>,
    mut player_query: Query<(Entity, &mut Stats), With<Player>>,
) {
    for ev in ev_gain_exp.read() {
        // 如果是 PLACEHOLDER，查找玩家实体
        let target_entity = if ev.entity == Entity::PLACEHOLDER {
            if let Ok((player_entity, _)) = player_query.single() {
                player_entity
            } else {
                continue;
            }
        } else {
            ev.entity
        };

        if let Ok((_, mut stats)) = player_query.get_mut(target_entity) {
            let old_level = stats.lv;
            stats.gain_exp(ev.amount);

            ev_log.write(LogEvent(format!("获得 {} 经验", ev.amount)));

            if stats.lv > old_level {
                ev_level_up.write(LevelUp {
                    entity: ev.entity,
                    new_level: stats.lv,
                });
                ev_log.write(LogEvent(format!(
                    "升级！等级 {} → {}，生命值 +2，攻击力 +1，防御力 +1",
                    old_level, stats.lv
                )));
            }
        }
    }
}

/// 处理受到伤害事件
pub fn handle_take_damage(
    mut ev_take_damage: EventReader<TakeDamage>,
    mut ev_death: EventWriter<Death>,
    mut ev_log: EventWriter<LogEvent>,
    mut player_query: Query<(Entity, &mut Stats), With<Player>>,
) {
    for ev in ev_take_damage.read() {
        // 如果是 PLACEHOLDER，查找玩家实体
        let target_entity = if ev.entity == Entity::PLACEHOLDER {
            if let Ok((player_entity, _)) = player_query.single() {
                player_entity
            } else {
                continue;
            }
        } else {
            ev.entity
        };

        if let Ok((_, mut stats)) = player_query.get_mut(target_entity) {
            let is_dead = stats.take_damage(ev.damage);

            ev_log.write(LogEvent(format!(
                "受到 {} 点伤害，当前生命值：{}/{}",
                ev.damage, stats.hp, stats.max_hp
            )));

            if is_dead {
                ev_death.write(Death {
                    entity: target_entity,
                });
                ev_log.write(LogEvent("死亡！".to_string()));
            }
        }
    }
}

/// 处理治疗事件
pub fn handle_heal(
    mut ev_heal: EventReader<Heal>,
    mut ev_log: EventWriter<LogEvent>,
    mut player_query: Query<(Entity, &mut Stats), With<Player>>,
) {
    for ev in ev_heal.read() {
        // 如果是 PLACEHOLDER，查找玩家实体
        let target_entity = if ev.entity == Entity::PLACEHOLDER {
            if let Ok((player_entity, _)) = player_query.single() {
                player_entity
            } else {
                continue;
            }
        } else {
            ev.entity
        };

        if let Ok((_, mut stats)) = player_query.get_mut(target_entity) {
            let old_hp = stats.hp;
            stats.heal(ev.amount);
            let healed = stats.hp - old_hp;

            if healed > 0 {
                ev_log.write(LogEvent(format!(
                    "恢复 {} 点生命值，当前生命值：{}/{}",
                    healed, stats.hp, stats.max_hp
                )));
            }
        }
    }
}

/// 处理装备变更事件 - 重新计算属性
pub fn handle_equipment_changed(
    mut ev_equipment_changed: EventReader<EquipmentChanged>,
    mut ev_recalculate: EventWriter<RecalculateStats>,
) {
    for ev in ev_equipment_changed.read() {
        ev_recalculate.write(RecalculateStats { entity: ev.entity });
    }
}

/// 重新计算属性（基础属性 + 装备加成）
pub fn recalculate_stats(
    mut ev_recalculate: EventReader<RecalculateStats>,
    mut stats_query: Query<(&mut Stats, &BaseStats)>,
    equipment: Res<Equipment>,
) {
    for ev in ev_recalculate.read() {
        if let Ok((mut stats, base_stats)) = stats_query.get_mut(ev.entity) {
            // 重置为基础属性
            let current_hp = stats.hp; // 保持当前血量
            stats.max_hp = base_stats.max_hp;
            stats.atk = base_stats.atk;
            stats.def = base_stats.def;
            stats.rng = base_stats.rng;

            // 应用装备加成
            apply_equipment_bonuses(&mut stats, &equipment);

            // 如果最大血量增加，按比例恢复血量
            if stats.max_hp > base_stats.max_hp {
                let hp_ratio = current_hp as f32 / base_stats.max_hp as f32;
                stats.hp = (stats.max_hp as f32 * hp_ratio).ceil() as i32;
                if stats.hp > stats.max_hp {
                    stats.hp = stats.max_hp;
                }
            } else {
                stats.hp = current_hp.min(stats.max_hp);
            }
        }
    }
}

/// 应用装备属性加成
fn apply_equipment_bonuses(stats: &mut Stats, equipment: &Equipment) {
    // 头部装备加成
    if let Some(head) = &equipment.head {
        stats.max_hp += head.proto.max_hp;
        stats.atk += head.proto.atk;
        stats.def += head.proto.def;
        stats.rng += head.proto.rng;
    }

    // 身体装备加成
    if let Some(body) = &equipment.body {
        stats.max_hp += body.proto.max_hp;
        stats.atk += body.proto.atk;
        stats.def += body.proto.def;
        stats.rng += body.proto.rng;
    }

    // 武器加成
    if let Some(weapon) = &equipment.weapon {
        stats.max_hp += weapon.proto.max_hp;
        stats.atk += weapon.proto.atk;
        stats.def += weapon.proto.def;
        stats.rng += weapon.proto.rng;
    }

    // 饰品加成
    if let Some(accessory) = &equipment.accessory {
        stats.max_hp += accessory.proto.max_hp;
        stats.atk += accessory.proto.atk;
        stats.def += accessory.proto.def;
        stats.rng += accessory.proto.rng;
    }
}

/// 显示属性信息
pub fn show_stats(
    mut ev_show_stats: EventReader<ShowStats>,
    mut ev_log: EventWriter<LogEvent>,
    player_query: Query<&Stats, With<Player>>,
    stats_query: Query<&Stats>,
) {
    for ev in ev_show_stats.read() {
        let stats = if let Some(entity) = ev.entity {
            stats_query.get(entity).ok()
        } else {
            player_query.single().ok()
        };

        if let Some(stats) = stats {
            ev_log.write(LogEvent(format!(
                "=== 角色属性 ===
生命值: {}/{}
攻击力: {}
防御力: {}
等级: {} (经验: {}/{})
攻击距离: {}
================",
                stats.hp,
                stats.max_hp,
                stats.atk,
                stats.def,
                stats.lv,
                stats.exp,
                stats.exp_to_next(),
                stats.rng
            )));
        } else {
            ev_log.write(LogEvent("未找到角色属性".to_string()));
        }
    }
}

/// 初始化玩家实体
pub fn spawn_player(mut commands: Commands) {
    let stats = Stats::default();
    let base_stats = BaseStats::from(&stats);

    commands.spawn((Player, stats, base_stats));
}
