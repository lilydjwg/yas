use anyhow::{Result, anyhow};

use crate::game_info::{GameInfo, Platform, UI, ResolutionFamily};
use crate::positioning::Rect;

pub fn get_game_info() -> Result<GameInfo> {
    let rect = Rect::new(0, 0, 1920, 1080);
    let rf = ResolutionFamily::new(rect.size()).ok_or(anyhow!("unknown resolution family"))?;

    Ok(GameInfo {
        window: rect.to_rect_i32(),
        resolution_family: rf,
        is_cloud: false,
        ui: UI::Desktop,
        platform: Platform::Windows,
    })
}
