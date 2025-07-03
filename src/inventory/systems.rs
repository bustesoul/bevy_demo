use bevy::prelude::*;
use crate::data::{ItemAssets, schema::ItemList};
use super::{components::*, events::*};

/// 处理"give"——往背包里塞 ItemStack
pub fn give_item(
    mut ev_give: EventReader<GiveItemEvent>,
    mut backpack: ResMut<Backpack>,
    item_assets: Res<ItemAssets>,
    lists: Res<Assets<ItemList>>,
) {
    let list = item_assets.handle
        .as_ref()
        .and_then(|h| lists.get(h))
        .expect("items must be loaded");

    for ev in ev_give.read() {
        if let Some(proto) = list.items
            .iter()
            .find(|e| e.id.eq_ignore_ascii_case(&ev.id))
        {
            // 查找是否已有同 ID 堆叠
            if let Some(stack) = backpack.slots
                .iter_mut()
                .find(|s| s.count > 0 && s.proto.id == proto.id)
            {
                stack.count += ev.count;
            } else {
                // 找空位
                if let Some(slot) = backpack.slots
                    .iter_mut()
                    .find(|s| s.count == 0)
                {
                    *slot = ItemStack { proto: proto.clone(), count: ev.count };
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
pub fn print_inventory(
    mut ev_list: EventReader<ListInventoryEvent>,
    backpack: Res<Backpack>,
) {
    if ev_list.is_empty() { return; }
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
