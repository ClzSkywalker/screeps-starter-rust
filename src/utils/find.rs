use std::{collections::HashMap, str::FromStr};

use screeps::{
    find, game,
    look::{self, PositionedLookResult},
    prelude::*,
    ConstructionSite, Creep, Position, Resource, ResourceType, Room, RoomName, Source,
    StructureController, StructureExtension, StructureObject, StructureRampart, StructureType,
    StructureWall, Tombstone,
};

use super::errorx::ScreepError;

/// 查询所有仍有能量的资源
///
/// * `room`:
pub fn find_source_all(room: &Room) -> Vec<Source> {
    let mut target: Vec<Source> = Vec::new();
    for source in room.find(find::SOURCES, None).iter() {
        target.push(source.clone());
    }
    target
}

pub fn find_controller(room: &Room) -> Option<StructureController> {
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureController(controller) = structure {
            if !controller.my() {
                continue;
            }
            return Some(controller.clone());
        }
    }
    None
}

pub fn find_rampart_all(room: &Room) -> Vec<StructureRampart> {
    let mut target: Vec<StructureRampart> = Vec::new();
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureRampart(site) = structure {
            target.push(site.clone());
        }
    }
    target
}

pub fn find_wall_all(room: &Room) -> Vec<StructureWall> {
    let mut target: Vec<StructureWall> = Vec::new();
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureWall(site) = structure {
            target.push(site.clone());
        }
    }
    target
}

pub fn find_ext_all(room: &Room) -> Vec<StructureExtension> {
    let mut target: Vec<StructureExtension> = Vec::new();
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureExtension(site) = structure {
            target.push(site.clone());
        }
    }
    target
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
/// * `range`: 查找资源点范围
#[derive(Default)]
pub struct FindStoreOption {
    pub resource_type: Option<ResourceType>,
    pub status: FindStoreStatus,
    pub withdraw: bool,
    pub ignore_structures: Vec<StructureType>,
    pub select_structures: Option<Vec<StructureType>>,
    pub priority: Option<Vec<StructureType>>,
    pub range: Option<usize>,
}

impl FindStoreOption {
    // 忽略spawn,extension,并且能够取出
    pub fn porter_up() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::UseCapacity,
            withdraw: true,
            ignore_structures: vec![
                StructureType::Spawn,
                StructureType::Extension,
                StructureType::Tower,
                StructureType::Storage,
            ],
            select_structures: Default::default(),
            priority: Some(vec![StructureType::Container]),
            range: None,
        }
    }

    pub fn porter_down() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::FreeCapacity,
            withdraw: false,
            ignore_structures: Default::default(),
            select_structures: Some(vec![
                StructureType::Spawn,
                StructureType::Extension,
                StructureType::Tower,
                StructureType::Storage,
            ]),
            priority: Some(vec![
                StructureType::Extension,
                StructureType::Spawn,
                StructureType::Tower,
                StructureType::Storage,
            ]),
            range: None,
        }
    }

    pub fn builder_up() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::UseCapacity,
            withdraw: true,
            ignore_structures: vec![
                StructureType::Spawn,
                StructureType::Extension,
                StructureType::Tower,
            ],
            select_structures: Default::default(),
            priority: Some(vec![StructureType::Storage, StructureType::Container]),
            range: None,
        }
    }

    pub fn repairer_down() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::FreeCapacity,
            withdraw: false,
            ignore_structures: Default::default(),
            select_structures: Some(vec![
                StructureType::Tower,
                StructureType::Road,
                StructureType::Wall,
            ]),
            priority: Some(vec![
                StructureType::Tower,
                StructureType::Road,
                StructureType::Wall,
            ]),
            range: None,
        }
    }

    pub fn spawn_ext_down() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::FreeCapacity,
            withdraw: false,
            ignore_structures: Default::default(),
            select_structures: Some(vec![
                StructureType::Spawn,
                StructureType::Extension,
                StructureType::Container,
            ]),
            priority: Some(vec![
                StructureType::Spawn,
                StructureType::Extension,
                StructureType::Container,
            ]),
            range: None,
        }
    }

    /// harvester 存储忽略ext,spawn
    pub fn harvester_store() -> Self {
        Self {
            resource_type: Some(ResourceType::Energy),
            status: FindStoreStatus::FreeCapacity,
            withdraw: false,
            ignore_structures: vec![StructureType::Spawn],
            select_structures: None,
            priority: None,
            range: Some(10),
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
            if let Some(range) = option.range {
                if !exist_range(creep.pos(), structure.pos(), range) {
                    continue;
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

/// 查询死亡后creep的遗物
///
/// * `creep`:
/// * `room`:
pub fn find_tombstone(creep: &Creep, room: &Room) -> Option<Tombstone> {
    let mut structure_list: Vec<Tombstone> = Vec::new();
    for structure in room.find(find::TOMBSTONES, None).iter() {
        if structure
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            > 0
        {
            structure_list.push(structure.clone());
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

pub fn get_extension_count(room: &Room) -> Vec<StructureExtension> {
    let mut structure_list: Vec<StructureExtension> = Vec::new();
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureExtension(s) = structure {
            structure_list.push(s.clone());
        }
    }
    structure_list
}

/// 需要修理的建筑物
///
/// * `room`:
pub fn find_need_repair(room: &Room) -> Vec<StructureObject> {
    room.find(find::STRUCTURES, None)
        .into_iter()
        .filter(|item| {
            let item = item.as_structure();
            item.hits_max() - item.hits() > 0
        })
        .collect()
}

/// 查询对应类型建筑物数目
///
/// * `room`:
/// * `s`:
// pub fn get_structure_count<T: find::FindConstant>(room: &Room, s: Vec<T>) -> usize {
//     let mut count = 0;
//     for ele in s {
//         count += room.find(ele, None).len();
//     }
//     count
// }

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
    let creep_pos = creep.pos();
    let creep_x = Position::x(creep_pos).u8();
    let creep_y = Position::y(creep_pos).u8();
    if let Some(structure) = structure_list.iter().min_by_key(|x| {
        let target_pos = x.pos();
        let target_x = Position::x(target_pos).u8();
        let target_y = Position::y(target_pos).u8();
        let x: f32 = if creep_x > target_x {
            f32::from(creep_x - target_x).powf(2.0)
        } else {
            f32::from(target_x - creep_x).powf(2.0)
        };

        let y: f32 = if creep_y > target_y {
            f32::from(creep_y - target_y).powf(2.0)
        } else {
            f32::from(target_y - creep_y).powf(2.0)
        };
        (x + y).sqrt() as i32
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
    let top_y = if y < range { 0 } else { y - range };
    let left_x = if x < range { 0 } else { x - range };
    let button_y = if y + range > 49 { 49 } else { y + range };
    let right_x = if x + range > 49 { 49 } else { x + range };
    room.look_for_at_area(look_type, top_y, left_x, button_y, right_x)
}

/// 判断两个位置是否在区间内
///
/// * `pos1`:
/// * `pos2`:
/// * `range`:
pub fn exist_range(pos1: Position, pos2: Position, range: usize) -> bool {
    let x1 = Position::x(pos1).u8();
    let y1 = Position::y(pos1).u8();
    let x2 = Position::x(pos2).u8();
    let y2 = Position::y(pos2).u8();
    let x = if x1 > x2 { x1 - x2 } else { x2 - x1 };
    let y = if y1 > y2 { y1 - y2 } else { y2 - y1 };
    let x = i32::from(x).pow(2);
    let y = i32::from(y).pow(2);
    ((x + y) as f32).sqrt() <= range as f32
}

/// 获取优先级高的建筑
///
/// * `struct_list`:
/// * `order`:
pub fn priority_structure(
    struct_list: Vec<StructureObject>,
    order: Vec<StructureType>,
) -> Vec<StructureObject> {
    let mut s_map: HashMap<StructureType, Vec<StructureObject>> = HashMap::new();
    for item in struct_list.iter() {
        if let Some(s) = s_map.get_mut(&item.structure_type()) {
            s.push(item.clone());
            continue;
        };
        s_map.insert(item.structure_type(), vec![item.clone()]);
    }
    for item in order {
        if let Some(r) = s_map.get(&item) {
            if item == StructureType::Wall {
                return r
                    .clone()
                    .into_iter()
                    .filter(|x| x.as_structure().hits() < 500000)
                    .collect();
            }
            return r.clone();
        }
    }
    struct_list
}

// pub fn priority_die(struct_list: &[StructureObject]) -> Option<StructureObject> {
//     if let Some(y) = struct_list.iter().min_by_key(|x| x.as_structure().hits()) {
//         return Some(y.clone());
//     };
//     None
// }

