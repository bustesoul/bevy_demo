use bevy::prelude::*;

/// 角色属性组件
#[derive(Component, Debug, Clone)]
pub struct Stats {
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub lv: i32,
    pub exp: i32,
    pub rng: i32,  // 基础攻击距离
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            hp: 20,
            max_hp: 20,
            atk: 2,
            def: 1,
            lv: 1,
            exp: 0,
            rng: 1,
        }
    }
}

impl Stats {
    /// 计算升级所需经验：exp_to_next = 10 × lv²
    pub fn exp_to_next(&self) -> i32 {
        10 * self.lv * self.lv
    }

    /// 检查是否可以升级
    pub fn can_level_up(&self) -> bool {
        self.exp >= self.exp_to_next()
    }

    /// 执行升级：+2 max_hp, +1 atk, +1 def
    pub fn level_up(&mut self) {
        if self.can_level_up() {
            self.exp -= self.exp_to_next();
            self.lv += 1;
            self.max_hp += 2;
            self.atk += 1;
            self.def += 1;
            // 升级时恢复满血
            self.hp = self.max_hp;
        }
    }

    /// 获得经验
    pub fn gain_exp(&mut self, amount: i32) {
        self.exp += amount;
        // 连续升级直到无法升级
        while self.can_level_up() {
            self.level_up();
        }
    }

    /// 受到伤害
    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.hp -= damage;
        if self.hp < 0 {
            self.hp = 0;
        }
        self.hp <= 0  // 返回是否死亡
    }

    /// 恢复生命值
    pub fn heal(&mut self, amount: i32) {
        self.hp += amount;
        if self.hp > self.max_hp {
            self.hp = self.max_hp;
        }
    }

    /// 检查是否死亡
    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }
}

/// 玩家标记组件
#[derive(Component)]
pub struct Player;

/// 基础属性（不受装备影响的原始属性）
#[derive(Component, Debug, Clone)]
pub struct BaseStats {
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub lv: i32,
    pub exp: i32,
    pub rng: i32,
}

impl Default for BaseStats {
    fn default() -> Self {
        Self {
            max_hp: 20,
            atk: 2,
            def: 1,
            lv: 1,
            exp: 0,
            rng: 1,
        }
    }
}

impl From<&Stats> for BaseStats {
    fn from(stats: &Stats) -> Self {
        Self {
            max_hp: stats.max_hp,
            atk: stats.atk,
            def: stats.def,
            lv: stats.lv,
            exp: stats.exp,
            rng: stats.rng,
        }
    }
}
