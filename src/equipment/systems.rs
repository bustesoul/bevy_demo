use super::{components::*, events::*};
use crate::character::events::EquipmentChanged;
use crate::core::events::LogEvent;
use crate::inventory::{components::Backpack, events::ListInventoryEvent};
use bevy::prelude::*;

pub fn equip_item(
    mut ev_equip: EventReader<EquipEvent>,
    mut equip: ResMut<Equipment>,
    mut backpack: ResMut<Backpack>,
    mut list_event: EventWriter<ListInventoryEvent>,
    mut equipment_changed: EventWriter<EquipmentChanged>,
    mut log_event: EventWriter<LogEvent>,
    player_query: Query<Entity, With<crate::character::components::Player>>,
) {
    for ev in ev_equip.read() {
        // 检查槽位是否有效
        if !Equipment::is_valid_slot(&ev.slot) {
            log_event.write(LogEvent(format!("未知装备槽: {}", ev.slot)));
            continue;
        }

        // 检查背包索引是否有效
        if ev.index >= backpack.slots.len() {
            log_event.write(LogEvent("背包索引超出范围".to_string()));
            continue;
        }

        // 先检查背包索引和物品
        let (item_proto, item_name) = {
            if let Some(stack) = backpack.slots.get(ev.index) {
                if stack.count == 0 {
                    log_event.write(LogEvent("该背包格为空".to_string()));
                    continue;
                }
                (stack.proto.clone(), stack.proto.name.clone())
            } else {
                continue;
            }
        };

        // 检查是否有旧装备需要放回背包
        let old_item = if let Some(slot_ref) = equip.get_slot_mut(&ev.slot) {
            slot_ref.take()
        } else {
            None
        };

        // 如果有旧装备，尝试放回背包
        if let Some(old_item) = old_item {
            if let Some(empty_slot) = backpack.slots.iter_mut().find(|s| s.count == 0) {
                *empty_slot = old_item;
            } else {
                log_event.write(LogEvent("背包已满，无法卸下原装备".to_string()));
                // 恢复原装备
                if let Some(slot_ref) = equip.get_slot_mut(&ev.slot) {
                    *slot_ref = Some(old_item);
                }
                continue;
            }
        }

        // 从背包中取出物品
        if let Some(stack) = backpack.slots.get_mut(ev.index) {
            stack.count -= 1;

            // 装备新物品
            let taken = crate::inventory::components::ItemStack {
                proto: item_proto,
                count: 1,
            };

            if let Some(slot_ref) = equip.get_slot_mut(&ev.slot) {
                *slot_ref = Some(taken);
            }

            log_event.write(LogEvent(format!("已装备 {}: {}", ev.slot, item_name)));

            // 触发装备变更事件
            if let Ok(player_entity) = player_query.single() {
                equipment_changed.write(EquipmentChanged {
                    entity: player_entity,
                });
            }

            // 刷新背包显示
            list_event.write(ListInventoryEvent);
        }
    }
}

/// 卸下装备
pub fn unequip_item(
    mut ev_unequip: EventReader<UnequipEvent>,
    mut equip: ResMut<Equipment>,
    mut backpack: ResMut<Backpack>,
    mut list_event: EventWriter<ListInventoryEvent>,
    mut equipment_changed: EventWriter<EquipmentChanged>,
    mut log_event: EventWriter<LogEvent>,
    player_query: Query<Entity, With<crate::character::components::Player>>,
) {
    for ev in ev_unequip.read() {
        // 检查槽位是否有效
        if !Equipment::is_valid_slot(&ev.slot) {
            log_event.write(LogEvent(format!("未知装备槽: {}", ev.slot)));
            continue;
        }

        if let Some(slot_ref) = equip.get_slot_mut(&ev.slot) {
            if let Some(item) = slot_ref.take() {
                // 尝试放回背包
                if let Some(empty_slot) = backpack.slots.iter_mut().find(|s| s.count == 0) {
                    *empty_slot = item.clone();
                    log_event.write(LogEvent(format!("已卸下 {}: {}", ev.slot, item.proto.name)));

                    // 触发装备变更事件
                    if let Ok(player_entity) = player_query.single() {
                        equipment_changed.write(EquipmentChanged {
                            entity: player_entity,
                        });
                    }

                    // 刷新背包显示
                    list_event.write(ListInventoryEvent);
                } else {
                    // 背包满了，恢复装备
                    *slot_ref = Some(item);
                    log_event.write(LogEvent("背包已满，无法卸下装备".to_string()));
                }
            } else {
                log_event.write(LogEvent(format!("{} 槽位为空", ev.slot)));
            }
        }
    }
}
