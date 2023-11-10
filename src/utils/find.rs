use screeps::{
    find, pathfinder::SingleRoomCostResult, prelude::*, ConstructionSite, Creep, FindPathOptions,
    ObjectId, Path, Room, RoomName, Source, StructureController, StructureObject, StructureType,
};

pub fn find_source(creep: &Creep, room: &Room) -> Option<ObjectId<Source>> {
    for item in room.find(find::SOURCES_ACTIVE, None).iter() {
        return Some(item.id());
    }
    None
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
    return None;
}

// status true有空间，false有存储
pub fn find_store(creep: &Creep, room: &Room, status: bool) -> Option<StructureObject> {
    let mut structure_list: Vec<StructureObject> = Vec::new();
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let Some(store) = structure.as_has_store() {
            if status && store.store().get_free_capacity(None) > 0 {
                continue;
            } else if store.store().get_used_capacity(None) == 0 {
                continue;
            }
            structure_list.push(structure.clone());
        }
    }
    let structure = structure_list.iter().min_by_key(|x| {
        let find_ops = FindPathOptions::<
            fn(RoomName, screeps::CostMatrix) -> SingleRoomCostResult,
            SingleRoomCostResult,
        >::new();
        let x = (*x).clone();
        match creep.pos().find_path_to(&x, Some(find_ops)) {
            Path::Vectorized(r) => r.len(),
            Path::Serialized(r) => r.len(),
        }
    });

    match structure {
        Some(r) => Some(r.clone()),
        None => None,
    }
}

pub fn find_site(room: &Room) -> Option<ConstructionSite> {
    for structure in room.find(find::CONSTRUCTION_SITES, None).iter() {
        // if !structure.my() {
        //     continue;
        // }
        return Some(structure.clone());
    }
    return None;
}
