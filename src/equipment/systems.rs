use bevy::prelude::*;
use crate::inventory::{components::Backpack, events::ListInventoryEvent};
use super::{components::*, events::*};

pub fn equip_item(
    mut ev_equip: EventReader<EquipEvent>,
    mut equip: ResMut<Equipment>,
    mut backpack: ResMut<Backpack>,
    mut list_event: EventWriter<ListInventoryEvent>,
) {
    for ev in ev_equip.read() {
        if ev.slot != "weapon" {
            println!("未知装备槽 {}", ev.slot);
            continue;
        }
        if let Some(stack) = backpack.slots.get_mut(ev.index) {
            if stack.count == 0 {
                println!("该背包格为空");
                continue;
            }
            // 取出 1 个
            let taken = crate::inventory::components::ItemStack { proto: stack.proto.clone(), count: 1 };
            stack.count -= 1;
            if stack.count == 0 { /* 留空位 */ }
            equip.weapon = Some(taken.clone());

            println!("已装备 weapon: {}", taken.proto.name);
            // 刷新背包显示
            list_event.write(ListInventoryEvent);
        }
    }
}
