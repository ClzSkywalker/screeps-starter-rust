use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use screeps::{
    constants::Terrain,
    find, game,
    look::{self},
    Creep, HasHits, HasPosition, HasTypedId, ObjectId, Position, Room, SharedCreepProperties,
    Source, StructureExtension, StructureRampart, StructureWall,
};
use serde::{Deserialize, Serialize};

use crate::utils::{self, errorx::ScreepError};

// creep 管理器
pub struct StructureManager {
    pub room_item: HashMap<String, RoomSourceItem>,
}

impl StructureManager {
    pub fn new() -> Self {
        StructureManager {
            room_item: HashMap::new(),
        }
    }

    pub fn init(&mut self, room_creeps: Vec<RoomSourceItem>) {
        for ele in room_creeps {
            self.room_item.insert(ele.room_id.clone(), ele);
        }
    }

    /// 发现并绑定资源
    ///
    /// * `room_id`:
    /// * `creep`:
    pub fn find_and_bind_source(
        &mut self,
        room_id: String,
        creep: &Creep,
    ) -> Option<ObjectId<Source>> {
        if let Some(source_i) = self.find_source_can_work(room_id.clone(), creep) {
            let s = source_i.id();
            if let Ok(r) = self.bind_source(room_id.clone(), s.to_string(), creep) {
                if r.is_some() {
                    return Some(s);
                }
            }
            log::info!("source not found");
            return None;
        }
        None
    }

    pub fn find_and_bind_rampart(
        &mut self,
        room_id: String,
        creep: &Creep,
    ) -> Option<screeps::RoomObject> {
        if let Some(source_i) = self.find_rampart_can_work(room_id.clone(), creep) {
            if let Ok(r) = self.bind_source(room_id.clone(), source_i.id().to_string(), creep) {
                if r.is_some() {
                    return Some(source_i.into());
                }
            }
            log::info!("source not found");
            return None;
        }
        None
    }

    /// 发现资源
    ///
    /// * `room_id`:
    /// * `creep`:
    pub fn find_source_can_work(&self, room_id: String, creep: &Creep) -> Option<Source> {
        let room = match utils::find::find_room(room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("{:?}", e);
                return None;
            }
        };
        let mut source_list = Vec::new();
        for source in room.find(find::SOURCES_ACTIVE, None).iter() {
            if !self.check_can_work(
                room_id.clone(),
                source.id().to_string(),
                creep.name().to_string(),
            ) {
                continue;
            }

            source_list.push(source.clone());
        }
        utils::find::get_near_site(creep, &source_list)
    }

    /// 发现可绑定的rampart
    ///
    /// * `room_id`:
    /// * `creep`:
    pub fn find_rampart_can_work(
        &self,
        room_id: String,
        creep: &Creep,
    ) -> Option<StructureRampart> {
        let room = match utils::find::find_room(room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("{:?}", e);
                return None;
            }
        };
        let mut site_list = Vec::new();

        for source in utils::find::find_rampart_all(&room).iter() {
            if !self.check_can_work(
                room_id.clone(),
                source.id().to_string(),
                creep.name().to_string(),
            ) {
                continue;
            }

            site_list.push(source.clone());
        }
        utils::find::get_near_site(creep, &site_list)
    }

    /// 绑定资源
    ///
    /// * `room_id`:
    /// * `source_id`:
    /// * `creep`:
    pub fn bind_source(
        &mut self,
        room_id: String,
        source_id: String,
        creep: &Creep,
    ) -> anyhow::Result<Option<()>> {
        match self.room_item.get_mut(&room_id) {
            Some(r) => {
                r.bind_screep(source_id.clone(), creep.name().to_string());
                Ok(Some(()))
            }
            None => {
                log::warn!("{:?}", ScreepError::RoomNotfound(room_id.clone()));
                Err(ScreepError::RoomNotfound(room_id.clone()).into())
            }
        }
    }

    pub fn cancel_bind(&mut self, room_id: String, creep_id: String) {
        match self.room_item.get_mut(&room_id) {
            Some(r) => {
                r.cancel_bind(creep_id);
            }
            None => {
                log::warn!("{:?}", ScreepError::RoomNotfound(room_id.clone()));
            }
        }
    }

    /// 检测该建筑是否能工作
    ///
    /// * `room_id`:
    /// * `source_id`:
    /// * `creep_id`:
    pub fn check_can_work(&self, room_id: String, source_id: String, creep_id: String) -> bool {
        if let Some(room) = self.room_item.get(&room_id) {
            return room.check_bind(source_id, creep_id);
        }
        false
    }

    /// 检测creep是否正常，把不正常的解除绑定
    pub fn check(&mut self) {
        for ele in self.room_item.values_mut() {
            ele.check();
        }
    }

    pub fn get_memory(&self, room_id: String) -> Option<RoomSourceItem> {
        for (rid, value) in self.room_item.iter() {
            if rid != &room_id {
                continue;
            }
            return Some(value.clone());
        }
        None
    }
}

impl Default for StructureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RoomSourceItem {
    pub room_id: String,
    pub source_max_map: HashMap<String, usize>, // 每个建筑最多绑定人数
    pub source_map: HashMap<String, StructureInfoEnum>,
}

impl RoomSourceItem {
    pub fn new(id: String) -> Self {
        RoomSourceItem {
            room_id: id,
            source_map: HashMap::new(),
            ..Default::default()
        }
    }

    /// 初始化结构，查询能够该source能够最多被多少个creep挖掘
    pub fn init(&mut self) -> anyhow::Result<()> {
        let mut source_count: usize = 0;
        let mut rampart_count: usize = 0;
        let mut wall_count: usize = 0;
        let mut ext_count: usize = 0;
        let room = match utils::find::find_room(self.room_id.clone()) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("{:?}", e);
                return Err(e);
            }
        };
        for ele in utils::find::find_source_all(&room) {
            let info = StructureInfoEnum::Source(StructureInfo::default()).init(
                &room,
                ele.id().to_string(),
                ele.pos(),
            );

            source_count += info.max_count;
            self.source_map.insert(ele.id().to_string(), info);
        }
        for ele in utils::find::find_rampart_all(&room) {
            let info = StructureInfoEnum::Rampart(StructureInfo::default()).init(
                &room,
                ele.id().to_string(),
                ele.pos(),
            );
            rampart_count += info.max_count;
            self.source_map.insert(ele.id().to_string(), info);
        }

        for ele in utils::find::find_wall_all(&room) {
            let info = StructureInfoEnum::Rampart(StructureInfo::default()).init(
                &room,
                ele.id().to_string(),
                ele.pos(),
            );
            wall_count += info.max_count;
            self.source_map.insert(ele.id().to_string(), info);
        }

        for ele in utils::find::find_ext_all(&room) {
            let info = StructureInfoEnum::Extension(StructureInfo::default()).init(
                &room,
                ele.id().to_string(),
                ele.pos(),
            );
            ext_count += info.max_count;
            self.source_map.insert(ele.id().to_string(), info);
        }
        self.source_max_map = HashMap::from([
            (
                StructureInfoEnum::Source(StructureInfo::default()).to_string(),
                source_count,
            ),
            (
                StructureInfoEnum::Rampart(StructureInfo::default()).to_string(),
                rampart_count,
            ),
            (
                StructureInfoEnum::Wall(StructureInfo::default()).to_string(),
                wall_count,
            ),
            (
                StructureInfoEnum::Extension(StructureInfo::default()).to_string(),
                ext_count,
            ),
        ]);

        Ok(())
    }

    /// 检测自身数据是否正常
    pub fn check(&mut self) {
        let mut vm_key = Vec::new();
        for (key, structure_info) in self.source_map.iter_mut() {
            if !structure_info.check_exist() {
                vm_key.push(key.clone());
                continue;
            }
            structure_info.check();
        }
        for ele in vm_key {
            self.source_map.remove(&ele);
        }
    }
    /// 检测能否进行绑定
    ///
    /// * `source_id`:
    /// * `creep_id`:
    pub fn check_bind(&self, source_id: String, creep_id: String) -> bool {
        if let Some(screeps) = self.source_map.get(&source_id) {
            if !screeps.work
                || (screeps.creeps.len() >= screeps.max_count
                    && !screeps.creeps.contains(&creep_id))
            {
                return false;
            }
            return true;
        }
        false
    }

    /// 绑定新关系，移除旧关系
    ///
    /// * `source_id`:
    /// * `creep_id`:
    pub fn bind_screep(&mut self, source_id: String, creep_id: String) {
        if let Some(screeps) = self.source_map.get_mut(&source_id) {
            if screeps.creeps.contains(&creep_id) {
                return;
            }
            screeps.creeps.push(creep_id.clone());
        }
        for (s_id, source_info) in self.source_map.iter_mut() {
            if s_id == &source_id {
                continue;
            }
            source_info.creeps.retain(|item| item != &creep_id);
        }
    }

    /// 取消绑定，当creep不再处理资源则取消绑定
    ///
    /// * `creep_id`:
    pub fn cancel_bind(&mut self, creep_id: String) {
        for (_, structure_info) in self.source_map.iter_mut() {
            structure_info.creeps.retain(|x| x != &creep_id);
        }
    }
}

#[derive(Debug, Clone, strum::Display, Serialize, Deserialize)]
pub enum StructureInfoEnum {
    #[strum(serialize = "source")]
    Source(StructureInfo),
    #[strum(serialize = "rampart")]
    Rampart(StructureInfo),
    #[strum(serialize = "wall")]
    Wall(StructureInfo),
    #[strum(serialize = "extension")]
    Extension(StructureInfo),
}

impl Deref for StructureInfoEnum {
    type Target = StructureInfo;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Source(s) => s,
            Self::Rampart(s) => s,
            Self::Wall(s) => s,
            Self::Extension(s) => s,
        }
    }
}

impl DerefMut for StructureInfoEnum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Source(s) => s,
            Self::Rampart(s) => s,
            Self::Wall(s) => s,
            Self::Extension(s) => s,
        }
    }
}

impl StructureInfoEnum {
    pub fn init(&self, room: &Room, structure_id: String, pos: Position) -> Self {
        match self {
            Self::Source(_) => {
                let count = utils::find::get_area_range(room, look::TERRAIN, pos, 1)
                    .iter()
                    .filter(|item| match item.look_result {
                        look::LookResult::Terrain(a) => {
                            if let Terrain::Wall = a {
                                return true;
                            }
                            false
                        }
                        _ => false,
                    })
                    .count();

                let mut max_count: usize = 9 - count;
                if max_count > 2 {
                    max_count = 2
                }

                let info = StructureInfo {
                    room_id: room.name().to_string(),
                    creeps: Vec::default(),
                    object_id: structure_id,
                    max_count,
                    work: true,
                };
                Self::Source(info)
            }
            Self::Rampart(_) => {
                let info = StructureInfo {
                    room_id: room.name().to_string(),
                    creeps: Vec::default(),
                    object_id: structure_id,
                    max_count: 1,
                    work: true,
                };
                Self::Rampart(info)
            }
            Self::Wall(_) => {
                let info = StructureInfo {
                    room_id: room.name().to_string(),
                    creeps: Vec::default(),
                    object_id: structure_id,
                    max_count: 1,
                    work: true,
                };
                Self::Wall(info)
            }
            Self::Extension(_) => {
                let info = StructureInfo {
                    room_id: room.name().to_string(),
                    creeps: Vec::default(),
                    object_id: structure_id,
                    max_count: 1,
                    work: true,
                };
                Self::Extension(info)
            }
        }
    }

    /// 检测数量是否正确，检测改建筑是否能够被工作
    pub fn check(&mut self) {
        match self {
            StructureInfoEnum::Source(s) => {
                s.creeps
                    .retain(|item| game::creeps().get(item.to_string()).is_some());

                let count = s.max_count;
                s.creeps.truncate(count);
                let id = s.object_id.as_str();
                let site_id: ObjectId<Source> = match ObjectId::from_str(id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, id);
                        return;
                    }
                };
                let site = match game::get_object_by_id_typed(&site_id) {
                    Some(r) => r,
                    None => {
                        log::warn!("get id none:{}", site_id);
                        return;
                    }
                };
                if site.energy() == 0 {
                    s.work = false;
                    s.creeps.clear();
                    return;
                }
                s.work = true;
            }
            StructureInfoEnum::Rampart(s) => {
                s.creeps
                    .retain(|item| game::creeps().get(item.to_string()).is_some());
                let id = s.object_id.as_str();
                let site_id: ObjectId<StructureRampart> = match ObjectId::from_str(id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, id);
                        return;
                    }
                };
                let site = match game::get_object_by_id_typed(&site_id) {
                    Some(r) => r,
                    None => {
                        log::warn!("get id none:{}", site_id);
                        return;
                    }
                };
                if site.hits() >= site.hits_max() {
                    s.work = false;
                    s.creeps.clear();
                    return;
                }
                s.work = true;
            }
            StructureInfoEnum::Wall(s) => {
                s.creeps
                    .retain(|item| game::creeps().get(item.to_string()).is_some());
                let id = s.object_id.as_str();
                let site_id: ObjectId<StructureWall> = match ObjectId::from_str(id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, id);
                        return;
                    }
                };
                let site = match game::get_object_by_id_typed(&site_id) {
                    Some(r) => r,
                    None => {
                        log::warn!("get id none:{}", site_id);
                        return;
                    }
                };
                if site.hits() >= site.hits_max() {
                    s.work = false;
                    s.creeps.clear();
                    return;
                }
                s.work = true;
            }
            StructureInfoEnum::Extension(s) => {
                s.creeps
                    .retain(|item| game::creeps().get(item.to_string()).is_some());
                let id = s.object_id.as_str();
                let site_id: ObjectId<StructureExtension> = match ObjectId::from_str(id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, id);
                        return;
                    }
                };
                let site = match game::get_object_by_id_typed(&site_id) {
                    Some(r) => r,
                    None => {
                        log::warn!("get id none:{}", site_id);
                        return;
                    }
                };
                if site.hits() >= site.hits_max() {
                    s.work = false;
                    s.creeps.clear();
                    return;
                }
                s.work = true;
            }
        }
    }

    pub fn check_exist(&mut self) -> bool {
        match self {
            StructureInfoEnum::Source(s) => {
                let site_id: ObjectId<Source> = match ObjectId::from_str(&s.object_id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, s.object_id);
                        return false;
                    }
                };
                game::get_object_by_id_typed(&site_id).is_some()
            }
            StructureInfoEnum::Rampart(s) => {
                let site_id: ObjectId<StructureRampart> = match ObjectId::from_str(&s.object_id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, s.object_id);
                        return false;
                    }
                };
                game::get_object_by_id_typed(&site_id).is_some()
            }
            StructureInfoEnum::Wall(s) => {
                let site_id: ObjectId<StructureWall> = match ObjectId::from_str(&s.object_id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, s.object_id);
                        return false;
                    }
                };
                game::get_object_by_id_typed(&site_id).is_some()
            }
            StructureInfoEnum::Extension(s) => {
                let site_id: ObjectId<StructureExtension> = match ObjectId::from_str(&s.object_id) {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("err:{},value:{}", e, s.object_id);
                        return false;
                    }
                };
                game::get_object_by_id_typed(&site_id).is_some()
            }
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructureInfo {
    pub room_id: String,
    pub object_id: String,
    pub creeps: Vec<String>, // 挖掘的creep id
    pub max_count: usize,    // 最大的工作挖掘人数
    pub work: bool,          // 该资源是否能正常工作
}

