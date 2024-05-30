use bevy::math::{IRect, IVec2};

use crate::{grid::Grid, area::Area};

pub trait Stainable: Grid {
    fn stained(&self) -> Area;
    fn stain(&mut self, area: IRect);
    fn stain_point(&mut self, point: IVec2);
    fn clear_stain(&mut self);

    fn stain_around(&mut self, point: IVec2, radius: i32) {
        self.stain(IRect::from_center_half_size(point, IVec2::splat(radius)))
    }
    
    fn is_stained(&self, point: IVec2) -> bool {
        self.stained().contains(point)
    }
}
