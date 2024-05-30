use std::{mem::replace, sync::Arc};

use bevy::math::IVec2;
use parking_lot::RwLock;

use crate::{cell::Cell, PowderkegError};

pub trait Grid {
    type Cell: Cell;

    fn get(&self, point: IVec2) -> Result<&Self::Cell, PowderkegError<Self::Cell>>;
    fn get_mut(&mut self, point: IVec2) ->Result<&mut Self::Cell, PowderkegError<Self::Cell>>;
    fn swap(&mut self, first: IVec2, second: IVec2) -> Result<(), PowderkegError<Self::Cell>>;

    fn get_state(&self, point: IVec2) -> Result<Arc<RwLock<<Self::Cell as Cell>::State>>, PowderkegError<Self::Cell>>;

    fn replace(&mut self, point: IVec2, cell: Self::Cell) -> Result<Self::Cell, PowderkegError<Self::Cell>> {
        Ok(replace(self.get_mut(point)?, cell))
    }

    fn map<T>(&self, point: IVec2, f: impl FnOnce(&Self::Cell) -> T) -> Result<T, PowderkegError<Self::Cell>> {
        self.get(point).map(f)
    }

    fn map_mut<T>(&mut self, point: IVec2, f: impl FnOnce(&mut Self::Cell) -> T) -> Result<T, PowderkegError<Self::Cell>> {
        self.get_mut(point).map(f)
    }

    fn at(&self, point: IVec2) -> &Self::Cell {
        self.get(point).unwrap_or_else(|e| panic!("error at {point}: {e}"))
    }

    fn at_mut(&mut self, point: IVec2) -> &mut Self::Cell {
        self.get_mut(point).unwrap_or_else(|e| panic!("error at {point}: {e}"))
    }

    fn state_at(&self, point: IVec2) -> Arc<RwLock<<Self::Cell as Cell>::State>> {
        self.get_state(point).unwrap_or_else(|e| panic!("error at {point}: {e}"))
    }
}
