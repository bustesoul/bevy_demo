use crate::inventory::components::ItemStack;
use bevy::prelude::*;

/// 装备系统 - 包含四个装备槽
#[derive(Resource, Default)]
pub struct Equipment {
    pub head: Option<ItemStack>,      // 头部装备
    pub body: Option<ItemStack>,      // 身体装备
    pub weapon: Option<ItemStack>,    // 武器
    pub accessory: Option<ItemStack>, // 饰品
}

impl Equipment {
    /// 获取指定槽位的装备
    pub fn get_slot(&self, slot: &str) -> Option<&ItemStack> {
        match slot {
            "head" => self.head.as_ref(),
            "body" => self.body.as_ref(),
            "weapon" => self.weapon.as_ref(),
            "accessory" => self.accessory.as_ref(),
            _ => None,
        }
    }

    /// 获取指定槽位的可变引用
    pub fn get_slot_mut(&mut self, slot: &str) -> Option<&mut Option<ItemStack>> {
        match slot {
            "head" => Some(&mut self.head),
            "body" => Some(&mut self.body),
            "weapon" => Some(&mut self.weapon),
            "accessory" => Some(&mut self.accessory),
            _ => None,
        }
    }

    /// 检查槽位名称是否有效
    pub fn is_valid_slot(slot: &str) -> bool {
        matches!(slot, "head" | "body" | "weapon" | "accessory")
    }

    /// 获取所有装备槽位名称
    pub fn all_slots() -> &'static [&'static str] {
        &["head", "body", "weapon", "accessory"]
    }
}
