use screeps::{Creep, ErrorCode, HasPosition, MoveToOptions, PolyStyle, SharedCreepProperties};

pub enum LineStatus {
    Harvesting,
    Building,
    Storing,
}

impl LineStatus {
    pub fn color(&self) -> &str {
        match self {
            LineStatus::Harvesting => "#34c724",
            LineStatus::Building => "#fff258",
            LineStatus::Storing => "#39a1e8",
        }
    }
}

pub fn route_option<T>(creep: &Creep, t: &T, status: LineStatus) -> Result<(), ErrorCode>
where
    T: HasPosition + AsRef<screeps::RoomObject>,
{
    match creep.move_to_with_options(
        t,
        Some(
            MoveToOptions::new().visualize_path_style(
                PolyStyle::default()
                    .line_style(screeps::LineDrawStyle::Solid)
                    .stroke(status.color()),
            ),
        ),
    ) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}
