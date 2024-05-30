use bevy::math::{IRect, IVec2};
use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};

#[derive(Debug, Clone)]
pub enum Area {
    Empty,
    Area(IRect),
    Many(Vec<IRect>),
}

impl Area {
    pub fn is_empty(&self) -> bool {
        match self {
            Area::Empty => true,
            _ => false,
        }
    }

    pub fn translate(&mut self, offset: IVec2) {
        match self {
            Area::Empty => {},
            Area::Area(area) => {
                area.min += offset;
                area.max += offset;
            },
            Area::Many(areas) => {
                for area in areas.iter_mut() {
                    area.min += offset;
                    area.max += offset;
                }
            },
        }
    }

    pub fn apply_randomly(&self, rng: &mut impl Rng, mut f: impl FnMut(IVec2)) {
        let mut choices: Vec<IVec2> = match self {
            Area::Empty => return,
            Area::Area(area) => {
                (area.min.y..=area.max.y)
                    .cartesian_product(area.min.x..=area.max.x)
                    .map(|(y, x)| IVec2::new(x, y)).collect()
            },
            Area::Many(areas) => {
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

        choices.as_mut_slice().shuffle(rng);

        for point in choices {
            f(point)
        }
    }

    pub fn apply(&self, mut f: impl FnMut(IVec2)) {
        match self {
            Area::Area(area) => {
                for y in area.min.y..=area.max.y {
                    for x in area.min.x..=area.max.x {
                        f(IVec2::new(x, y))
                    }
                }
            },
            Area::Many(areas) => {
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

    pub fn contains(&self, point: IVec2) -> bool {
        match self {
            Area::Empty => false,
            Area::Area(area) => area.min.x <= point.x && point.x <= area.max.x && area.min.y <= point.y && point.y <= area.max.y,
            Area::Many(areas) => areas.iter().any(|area| area.min.x <= point.x && point.x <= area.max.x && area.min.y <= point.y && point.y <= area.max.y),
        }
    }

    pub fn from_areas(stains: impl Iterator<Item = Self>) -> Self {
        let mut final_stains = Vec::new();

        for stain in stains {
            match stain {
                Area::Area(area) => final_stains.push(area),
                Area::Many(areas) => final_stains.extend(areas),
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

impl From<Option<IRect>> for Area {
    fn from(value: Option<IRect>) -> Self {
        match value {
            Some(area) => Self::Area(area),
            None => Self::Empty,
        }
    }
}

impl From<IRect> for Area {
    fn from(area: IRect) -> Self {
        Self::Area(area)
    }
}