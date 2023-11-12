use std::str::FromStr;

use screeps::{
    find, game, pathfinder::SingleRoomCostResult, prelude::*, ConstructionSite, Creep,
    FindPathOptions, ObjectId, Path, Room, RoomName, Source, StructureController, StructureObject,
};

use super::errorx::ScreepError;

pub fn find_source_all(room: &Room) -> Vec<Source> {
    let mut target: Vec<Source> = Vec::new();
    for source in room.find(find::SOURCES_ACTIVE, None).iter() {
        target.push(source.clone());
    }
    target
}

pub fn find_controller(room: &Room) -> Option<ObjectId<StructureController>> {
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureController(controller) = structure {
            if !controller.my() {
                continue;
            }
            return Some(controller.id());
        }
    }
    None
}

// status true有空间，false有存储
pub fn find_store(creep: &Creep, room: &Room, status: bool) -> Option<StructureObject> {
    let mut structure_list: Vec<StructureObject> = Vec::new();
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let Some(store1) = structure.as_has_store() {
            if (status && store1.store().get_free_capacity(None) == 0)
                || (!status && store1.store().get_used_capacity(None) == 0)
            {
                continue;
            }
            if structure.as_withdrawable().is_some() {
                structure_list.push(structure.clone());
            }
        }
    }
    get_near_site(creep, &structure_list)
}

pub fn find_site(creep: &Creep, room: &Room) -> Option<ConstructionSite> {
    let mut structure_list: Vec<ConstructionSite> = Vec::new();
    for structure in room.find(find::MY_CONSTRUCTION_SITES, None).iter() {
        structure_list.push(structure.clone());
        // return Some(structure.clone());
    }
    get_near_site(creep, &structure_list)
}

pub fn find_room(name: String) -> anyhow::Result<Room> {
    let room = match RoomName::from_str(&name) {
        Ok(r) => r,
        Err(e) => {
            return Err(e.into());
        }
    };

    let room = match game::rooms().get(room) {
        Some(r) => r,
        None => return Err(ScreepError::RoomNotfound(name.clone()).into()),
    };
    Ok(room)
}

// 寻找离creep最近的建筑物
pub fn get_near_site<T>(creep: &Creep, structure_list: &[T]) -> Option<T>
where
    T: Clone + HasPosition,
{
    if let Some(structure) = structure_list.iter().min_by_key(|x| {
        let find_ops = FindPathOptions::<
            fn(RoomName, screeps::CostMatrix) -> SingleRoomCostResult,
            SingleRoomCostResult,
        >::new();
        let x = (*x).clone();
        match creep.pos().find_path_to(&x, Some(find_ops)) {
            Path::Vectorized(r) => r.len(),
            Path::Serialized(r) => r.len(),
        }
    }) {
        return Some(structure.clone());
    };
    None
}
