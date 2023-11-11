use std::collections::HashSet;

use screeps::game;

pub mod errorx;
pub mod find;
pub mod line;

pub fn remove_expire_screep(param: &mut Vec<String>) {
    param.retain(|x| {
        if game::creeps().get(x.to_string()).is_some() {
            return true;
        }
        false
    });
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
