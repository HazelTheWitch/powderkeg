use std::mem::replace;

use bevy::math::IVec2;

pub trait Grid {
    type Cell;
    type State;

    fn get(&self, point: IVec2) -> Option<&Self::Cell>;
    fn get_mut(&mut self, point: IVec2) ->Option<&mut Self::Cell>;
    fn swap(&mut self, first: IVec2, second: IVec2) -> Option<()>;

    fn get_state(&self, point: IVec2) -> Option<&Self::State>;
    fn get_state_mut(&mut self, point: IVec2) -> Option<&mut Self::State>;

    fn replace(&mut self, point: IVec2, cell: Self::Cell) -> Option<Self::Cell> {
        Some(replace(self.get_mut(point)?, cell))
    }

    fn map<T>(&self, point: IVec2, f: impl FnOnce(&Self::Cell) -> T) -> Option<T> {
        self.get(point).map(f)
    }

    fn map_mut<T>(&mut self, point: IVec2, f: impl FnOnce(&mut Self::Cell) -> T) -> Option<T> {
        self.get_mut(point).map(f)
    }

    fn at(&self, point: IVec2) -> &Self::Cell {
        self.get(point).unwrap_or_else(|| panic!("{point} out of bounds"))
    }

    fn at_mut(&mut self, point: IVec2) -> &mut Self::Cell {
        self.get_mut(point).unwrap_or_else(|| panic!("{point} out of bounds"))
    }
}
