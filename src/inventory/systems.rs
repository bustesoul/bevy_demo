use super::{components::*, events::*};
use crate::character::events::{GainExp, Heal};
use crate::core::events::LogEvent;
use crate::data::{ItemAssets, schema::ItemList};
use bevy::prelude::*;

/// 处理"give"——往背包里塞 ItemStack
pub fn give_item(
    mut ev_give: EventReader<GiveItemEvent>,
    mut backpack: ResMut<Backpack>,
    item_assets: Res<ItemAssets>,
    lists: Res<Assets<ItemList>>,
) {
    let list = item_assets
        .handle
        .as_ref()
        .and_then(|h| lists.get(h))
        .expect("items must be loaded");

    for ev in ev_give.read() {
        if let Some(proto) = list
            .items
            .iter()
            .find(|e| e.id.eq_ignore_ascii_case(&ev.id))
        {
            // 查找是否已有同 ID 堆叠
            if let Some(stack) = backpack
                .slots
                .iter_mut()
                .find(|s| s.count > 0 && s.proto.id == proto.id)
            {
                stack.count += ev.count;
            } else {
                // 找空位
                if let Some(slot) = backpack.slots.iter_mut().find(|s| s.count == 0) {
                    *slot = ItemStack {
                        proto: proto.clone(),
                        count: ev.count,
                    };
                } else {
                    warn!("背包已满，无法获得 {}", proto.name);
                }
            }
            info!("获得 {} ×{}", proto.name, ev.count);
        } else {
            warn!("不存在物品 ID {}", ev.id);
        }
    }
}

/// 打印背包内容
pub fn print_inventory(mut ev_list: EventReader<ListInventoryEvent>, backpack: Res<Backpack>) {
    if ev_list.is_empty() {
        return;
    }
    ev_list.clear();

    let mut empty = true;
    for (idx, stack) in backpack.slots.iter().enumerate() {
        if stack.count > 0 {
            empty = false;
            println!(
                "[{idx}] {} ×{} (id={})",
                stack.proto.name, stack.count, stack.proto.id
            );
        }
    }

    if empty {
        println!("  (empty)");
    }
}

/// 使用物品
pub fn use_item(
    mut ev_use: EventReader<UseItemEvent>,
    mut backpack: ResMut<Backpack>,
    mut log_event: EventWriter<LogEvent>,
    mut heal_event: EventWriter<Heal>,
    mut _gain_exp_event: EventWriter<GainExp>,
    player_query: Query<Entity, With<crate::character::components::Player>>,
) {
    for ev in ev_use.read() {
        if ev.index >= backpack.slots.len() {
            log_event.write(LogEvent("背包索引超出范围".to_string()));
            continue;
        }

        if let Some(stack) = backpack.slots.get_mut(ev.index) {
            if stack.count == 0 {
                log_event.write(LogEvent("该背包格为空".to_string()));
                continue;
            }

            let item = &stack.proto;
            let item_name = item.name.clone();

            // 根据物品类型执行不同的使用效果
            match item.item_type.as_str() {
                "potion" => {
                    if item.heal > 0 {
                        log_event.write(LogEvent(format!(
                            "使用 {}，恢复 {} 点生命值",
                            item_name, item.heal
                        )));
                        if let Ok(player_entity) = player_query.single() {
                            heal_event.write(Heal {
                                entity: player_entity,
                                amount: item.heal,
                            });
                        }
                    } else {
                        log_event.write(LogEvent(format!("使用 {}，但没有任何效果", item_name)));
                    }
                }
                "scroll" => {
                    log_event.write(LogEvent(format!("使用 {}，获得临时增益效果", item_name)));
                    // TODO: 实现 Buff 系统
                }
                "key" => {
                    log_event.write(LogEvent(format!(
                        "使用 {}，但这里没有门可以开启",
                        item_name
                    )));
                    // TODO: 实现门锁系统
                    continue; // 钥匙不消耗
                }
                _ => {
                    log_event.write(LogEvent(format!("{} 无法使用", item_name)));
                    continue; // 不消耗物品
                }
            }

            // 消耗物品
            stack.count -= 1;
            if stack.count == 0 {
                // 清空槽位
                *stack = ItemStack {
                    proto: Default::default(),
                    count: 0,
                };
            }
        }
    }
}
