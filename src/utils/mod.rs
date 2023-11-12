use std::collections::HashSet;

use screeps::game;

pub mod errorx;
pub mod find;
pub mod line;

pub fn remove_expire_screep(param: &mut Vec<String>) {
    param.retain(|x| check_creep(x.clone()));
}

pub fn remove_repeat_screep(param: &mut Vec<String>) {
    let mut h = HashSet::new();
    param.retain(|x| {
        if h.contains(x) {
            return false;
        }
        h.insert(x.clone());
        true
    });
}

pub fn check_creep(name: String) -> bool {
    if let Some(creep) = game::creeps().get(name) {
        if let Some(live) = creep.ticks_to_live() {
            return live > 0;
        }
    }
    false
}
