use bevy::math::{IRect, IVec2};
use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};

use crate::grid::Grid;

#[derive(Debug, Clone)]
pub enum Stain {
    Empty,
    Area(IRect),
    Many(Vec<IRect>),
}

impl Stain {
    pub fn is_empty(&self) -> bool {
        match self {
            Stain::Empty => true,
            Stain::Area(area) => area.is_empty(),
            Stain::Many(areas) => areas.iter().all(|area| area.is_empty()),
        }
    }

    pub fn apply_randomly(&self, mut f: impl FnMut(IVec2)) {
        let mut choices: Vec<IVec2> = match self {
            Stain::Empty => return,
            Stain::Area(area) => {
                (area.min.y..=area.max.y)
                    .cartesian_product(area.min.x..=area.max.x)
                    .map(|(y, x)| IVec2::new(x, y)).collect()
            },
            Stain::Many(areas) => {
                areas
                    .iter()
                    .flat_map(|area| 
                        (area.min.y..=area.max.y)
                            .cartesian_product(area.min.x..=area.max.x)
                            .map(|(y, x)| IVec2::new(x, y))
                    )
                    .collect()
            },
        };

        choices.as_mut_slice().shuffle(&mut thread_rng());

        for point in choices {
            f(point)
        }
    }

    pub fn apply(&self, mut f: impl FnMut(IVec2)) {
        match self {
            Stain::Area(area) => {
                for y in area.min.y..=area.max.y {
                    for x in area.min.x..=area.max.x {
                        f(IVec2::new(x, y))
                    }
                }
            },
            Stain::Many(areas) => {
                for area in areas.iter() {
                    for y in area.min.y..=area.max.y {
                        for x in area.min.x..=area.max.x {
                            f(IVec2::new(x, y))
                        }
                    }
                }
            },
            _ => {},
        }
    }

    pub fn from_stains(stains: impl Iterator<Item = Self>) -> Self {
        let mut final_stains = Vec::new();

        for stain in stains {
            match stain {
                Stain::Area(area) => final_stains.push(area),
                Stain::Many(areas) => final_stains.extend(areas),
                _ => {},
            }
        }

        if final_stains.len() == 0 {
            Self::Empty
        } else {
            Self::Many(final_stains)
        }
    }
}

impl From<Option<IRect>> for Stain {
    fn from(value: Option<IRect>) -> Self {
        match value {
            Some(area) => Self::from(area),
            None => Self::Empty,
        }
    }
}

impl From<IRect> for Stain {
    fn from(area: IRect) -> Self {
        if area.is_empty() {
            Self::Empty
        } else {
            Self::Area(area)
        }
    }
}

impl Stain {
    pub fn contains(&self, point: IVec2) -> bool {
        match self {
            Stain::Empty => false,
            Stain::Area(area) => area.contains(point),
            Stain::Many(areas) => areas.iter().any(|area| area.contains(point)),
        }
    }
}

pub trait Stainable: Grid {
    fn stained(&self) -> Stain;
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
