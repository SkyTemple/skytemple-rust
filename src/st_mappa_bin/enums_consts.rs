/*
 * Copyright 2021-2022 Capypara and the SkyTemple Contributors
 *
 * This file is part of SkyTemple.
 *
 * SkyTemple is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * SkyTemple is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with SkyTemple.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::python::*;
use packed_struct::prelude::*;

pub const GUARANTEED: u16 = 0xFFFF;

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "python", derive(EnumToPy_u16))]
pub enum Probability {
    Percentage(u16), // as fixed int.
    Guaranteed,
}

impl From<Probability> for u16 {
    fn from(prob: Probability) -> Self {
        match prob {
            Probability::Percentage(v) => v,
            Probability::Guaranteed => GUARANTEED,
        }
    }
}

impl From<u16> for Probability {
    fn from(v: u16) -> Self {
        match v {
            GUARANTEED => Probability::Guaranteed,
            vv => Probability::Percentage(vv),
        }
    }
}

impl PrimitiveEnum for Probability {
    type Primitive = u16;

    fn from_primitive(val: Self::Primitive) -> Option<Self> {
        Some(val.into())
    }

    fn to_primitive(&self) -> Self::Primitive {
        (*self).into()
    }

    fn from_str(_s: &str) -> Option<Self> {
        // Not available for Probability.
        None
    }

    fn from_str_lower(_s: &str) -> Option<Self> {
        // Not available for Probability.
        None
    }
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum MappaFloorStructureType {
    MediumLarge = 0,
    Small = 1,
    SingleMonsterHouse = 2,
    Ring = 3,
    Crossroads = 4,
    TwoRoomsOneMonsterHouse = 5,
    Line = 6,
    Cross = 7,
    SmallMedium = 8,
    Beetle = 9,
    OuterRooms = 10,
    Medium = 11,
    MediumLarge12 = 12,
    MediumLarge13 = 13,
    MediumLarge14 = 14,
    MediumLarge15 = 15,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum MappaFloorWeather {
    Clear = 0,
    Sunny = 1,
    Sandstorm = 2,
    Cloudy = 3,
    Rainy = 4,
    Hail = 5,
    Fog = 6,
    Snow = 7,
    Random = 8,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum MappaFloorDarknessLevel {
    NoDarkness = 0,
    HeavyDarkness = 1,
    LightDarkness = 2,
    ThreeTile = 3,
    FourTile = 4,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum MappaTrapType {
    Unused = 0,
    MudTrap = 1,
    StickyTrap = 2,
    GrimyTrap = 3,
    SummonTrap = 4,
    PitfallTrap = 5,
    WarpTrap = 6,
    GustTrap = 7,
    SpinTrap = 8,
    SlumberTrap = 9,
    SlowTrap = 10,
    SealTrap = 11,
    PoisonTrap = 12,
    SelfdestructTrap = 13,
    ExplosionTrap = 14,
    PpZeroTrap = 15,
    ChestnutTrap = 16,
    WonderTile = 17,
    MonsterTrap = 18,
    SpikedTile = 19,
    StealthRock = 20,
    ToxicSpikes = 21,
    TripTrap = 22,
    RandomTrap = 23,
    GrudgeTrap = 24,
}
