use std::{collections::HashMap, str::FromStr};

use screeps::{
    find, game,
    look::{self, PositionedLookResult},
    pathfinder::SingleRoomCostResult,
    prelude::*,
    ConstructionSite, Creep, FindPathOptions, ObjectId, Path, Position, Resource, ResourceType,
    Room, RoomName, Source, StructureController, StructureObject, StructureType,
};

use super::errorx::ScreepError;

/// 查询所有仍有能量的资源
///
/// * `room`:
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

#[derive(Default)]
pub enum FindStoreStatus {
    #[default]
    Default,
    // 有可用来存储的空间
    FreeCapacity,
    // 有可以使用的资源
    UseCapacity,
}

///
/// 寻找存储建筑物的参数
/// * `resource_type`:
/// * `status`: true有空间，false有存储
/// * `withdraw`: 是否包含可拾取条件
/// * `ignore_structures`: 需要屏蔽的建筑
/// * `select_structures`: 指定建筑类型
/// *  priority: 优先级
#[derive(Default)]
pub struct FindStoreOption {
    pub resource_type: Option<ResourceType>,
    pub status: FindStoreStatus,
    pub withdraw: bool,
    pub ignore_structures: Vec<StructureType>,
    pub select_structures: Option<Vec<StructureType>>,
    pub priority: Option<Vec<StructureType>>,
}

impl FindStoreOption {
    // 忽略spawn,extension,并且能够取出
    pub fn carry_up() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::UseCapacity,
            withdraw: true,
            ignore_structures: vec![StructureType::Spawn, StructureType::Extension],
            select_structures: Default::default(),
            priority: None,
        }
    }

    pub fn carry_down() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::FreeCapacity,
            withdraw: false,
            ignore_structures: Default::default(),
            select_structures: Some(vec![
                StructureType::Spawn,
                StructureType::Extension,
                StructureType::Storage,
            ]),
            priority: Some(vec![
                StructureType::Extension,
                StructureType::Spawn,
                StructureType::Storage,
            ]),
        }
    }

    /// harvester 存储忽略ext,spawn
    pub fn harvester_build() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::FreeCapacity,
            withdraw: false,
            ignore_structures: vec![StructureType::Spawn, StructureType::Extension],
            select_structures: None,
            priority: None,
        }
    }
}

/// 查询存储资源的容器建筑
///
/// * `creep`:
/// * `room`:
/// * `option`:
pub fn find_store(
    creep: &Creep,
    room: &Room,
    option: Option<FindStoreOption>,
) -> Option<StructureObject> {
    let mut structure_list: Vec<StructureObject> = Vec::new();
    let option = option.unwrap_or_default();
    for structure in room.find(find::STRUCTURES, None).iter() {
        if !structure.is_active() {
            continue;
        }
        if option
            .ignore_structures
            .contains(&structure.structure_type())
        {
            continue;
        }

        if let Some(select) = &option.select_structures {
            if !select.contains(&structure.structure_type()) {
                continue;
            }
        }

        if let Some(store1) = structure.as_has_store() {
            match option.status {
                FindStoreStatus::Default => {}
                FindStoreStatus::FreeCapacity => {
                    if store1.store().get_free_capacity(option.resource_type) == 0 {
                        continue;
                    }
                }
                FindStoreStatus::UseCapacity => {
                    if store1.store().get_used_capacity(option.resource_type) == 0 {
                        continue;
                    }
                }
            }
            if !option.withdraw {
                structure_list.push(structure.clone());
                continue;
            }
            if structure.as_withdrawable().is_some() {
                structure_list.push(structure.clone());
            }
        }
    }
    if let Some(priority) = option.priority {
        let mut pri_map: HashMap<StructureType, Vec<StructureObject>> = HashMap::new();
        for ele in structure_list.iter() {
            if let Some(v) = pri_map.get_mut(&ele.structure_type()) {
                v.push(ele.clone());
                continue;
            }
            pri_map.insert(ele.structure_type(), vec![ele.clone()]);
        }

        for item in priority {
            if let Some(values) = pri_map.get(&item) {
                structure_list = values.clone();
                break;
            }
        }
    }
    get_near_site(creep, &structure_list)
}

/// 查询掉落的资源
///
/// * `creep`:
/// * `room`:
pub fn find_drop_resource(creep: &Creep, room: &Room) -> Option<Resource> {
    let mut structure_list: Vec<Resource> = Vec::new();
    for structure in room.find(find::DROPPED_RESOURCES, None).iter() {
        structure_list.push(structure.clone());
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

/// 根据房间名查询房间
///
/// * `name`:
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

pub fn get_area_range<T: look::LookConstant>(
    room: &Room,
    look_type: T,
    pos: Position,
    range: u8,
) -> Vec<PositionedLookResult> {
    let x = Position::x(pos).u8();
    let y = Position::y(pos).u8();
    room.look_for_at_area(look_type, y - range, x - range, y + range, x + range)
}

