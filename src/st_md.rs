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
use crate::bytes::StBytes;
use crate::err::convert_packing_err;
use crate::python::*;
use bytes::Buf;
use packed_struct::prelude::*;
use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::mem::size_of;
use std::ops::Deref;
use std::sync::Mutex;
use std::vec;

const DEFAULT_NUM_ENTITIES: u32 = 600;
const DEFAULT_MAX_POSSIBLE: u32 = 554;
static mut MD_PROPERTIES_STATE_INSTANCE: Mutex<Option<Py<MdPropertiesState>>> = Mutex::new(None);

#[pyclass(module = "skytemple_rust.st_md")]
#[derive(Clone)]
struct MdPropertiesState {
    #[pyo3(get, set)]
    num_entities: u32,
    #[pyo3(get, set)]
    max_possible: u32,
}

impl MdPropertiesState {
    pub fn instance(py: Python) -> PyResult<Py<Self>> {
        let inst_mutex = unsafe { &mut MD_PROPERTIES_STATE_INSTANCE };
        let inst_locked = inst_mutex.get_mut().unwrap();
        if inst_locked.is_none() {
            *inst_locked = Some(Py::new(
                py,
                MdPropertiesState {
                    num_entities: DEFAULT_NUM_ENTITIES,
                    max_possible: DEFAULT_MAX_POSSIBLE,
                },
            )?)
        }
        Ok(inst_locked.deref().as_ref().unwrap().clone())
    }
}

#[pymethods]
impl MdPropertiesState {
    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "instance")]
    pub fn _instance(_cls: &PyType, py: Python) -> PyResult<Py<Self>> {
        Self::instance(py)
    }
}

#[derive(PrimitiveEnum_u16, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u16))]
pub enum EvolutionMethod {
    None = 0,
    Level = 1,
    Iq = 2,
    Items = 3,
    Recruited = 4,
    NoReq = 5,
}

#[derive(PrimitiveEnum_u16, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u16))]
pub enum AdditionalRequirement {
    None = 0,
    LinkCable = 1,
    AtkGDef = 2,
    AtkLDef = 3,
    AtkEDef = 4,
    SunRibbon = 5,
    LunarRibbon = 6,
    BeautyScarf = 7,
    IntVal1 = 8,
    IntVal0 = 9,
    Male = 10,
    Female = 11,
    AncientPower = 12,
    Rollout = 13,
    DoubleHit = 14,
    Mimic = 15,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum Gender {
    Invalid = 0,
    Male = 1,
    Female = 2,
    Genderless = 3,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum PokeType {
    None = 0,
    Normal = 1,
    Fire = 2,
    Water = 3,
    Grass = 4,
    Electric = 5,
    Ice = 6,
    Fighting = 7,
    Poison = 8,
    Ground = 9,
    Flying = 10,
    Psychic = 11,
    Bug = 12,
    Rock = 13,
    Ghost = 14,
    Dragon = 15,
    Dark = 16,
    Steel = 17,
    Neutral = 18,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum MovementType {
    Standard = 0,
    Unknown1 = 1,
    Hovering = 2,
    PhaseThroughWalls = 3,
    Lava = 4,
    Water = 5,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum IQGroup {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    Unused8 = 8,
    Unused9 = 9,
    I = 0xA,
    J = 0xB,
    UnusedC = 0xC,
    UnusedD = 0xD,
    UnusedE = 0xE,
    Invalid = 0xF,
}

#[derive(PrimitiveEnum_u8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_u8))]
pub enum Ability {
    Stench = 0x1,
    ThickFat = 0x2,
    RainDish = 0x3,
    Drizzle = 0x4,
    ArenaTrap = 0x5,
    Intimidate = 0x6,
    RockHead = 0x7,
    AirLock = 0x8,
    HyperCutter = 0x9,
    ShadowTag = 0xA,
    SpeedBoost = 0xB,
    BattleArmor = 0xC,
    Sturdy = 0xD,
    SuctionCups = 0xE,
    ClearBody = 0xF,
    Torrent = 0x10,
    Guts = 0x11,
    RoughSkin = 0x12,
    ShellArmor = 0x13,
    NaturalCure = 0x14,
    Damp = 0x15,
    Limber = 0x16,
    MagnetPull = 0x17,
    WhiteSmoke = 0x18,
    Synchronize = 0x19,
    Overgrow = 0x1A,
    SwiftSwim = 0x1B,
    SandStream = 0x1C,
    SandVeil = 0x1D,
    KeenEye = 0x1E,
    InnerFocus = 0x1F,
    Static = 0x20,
    ShedSkin = 0x21,
    HugePower = 0x22,
    VoltAbsorb = 0x23,
    WaterAbsorb = 0x24,
    Forecast = 0x25,
    SereneGrace = 0x26,
    PoisonPoint = 0x27,
    Trace = 0x28,
    Oblivious = 0x29,
    Truant = 0x2A,
    RunAway = 0x2B,
    StickyHold = 0x2C,
    CloudNine = 0x2D,
    Illuminate = 0x2E,
    EarlyBird = 0x2F,
    Hustle = 0x30,
    Drought = 0x31,
    LightningRod = 0x32,
    CompoundEyes = 0x33,
    MarvelScale = 0x34,
    WonderGuard = 0x35,
    Insomnia = 0x36,
    Levitate = 0x37,
    Plus = 0x38,
    Pressure = 0x39,
    LiquidOoze = 0x3A,
    ColorChange = 0x3B,
    Soundproof = 0x3C,
    EffectSpore = 0x3D,
    FlameBody = 0x3E,
    Minus = 0x3F,
    OwnTempo = 0x40,
    MagmaArmor = 0x41,
    WaterVeil = 0x42,
    Swarm = 0x43,
    CuteCharm = 0x44,
    Immunity = 0x45,
    Blaze = 0x46,
    Pickup = 0x47,
    FlashFire = 0x48,
    VitalSpirit = 0x49,
    Chlorophyll = 0x4A,
    PurePower = 0x4B,
    ShieldDust = 0x4C,
    IceBody = 0x4D,
    Stall = 0x4E,
    AngerPoint = 0x4F,
    TintedLens = 0x50,
    Hydration = 0x51,
    Frisk = 0x52,
    MoldBreaker = 0x53,
    Unburden = 0x54,
    DrySkin = 0x55,
    Anticipation = 0x56,
    Scrappy = 0x57,
    SuperLuck = 0x58,
    Gluttony = 0x59,
    SolarPower = 0x5A,
    SkillLink = 0x5B,
    Reckless = 0x5C,
    Sniper = 0x5D,
    SlowStart = 0x5E,
    Heatproof = 0x5F,
    Download = 0x60,
    Simple = 0x61,
    TangledFeet = 0x62,
    Adaptability = 0x63,
    Technician = 0x64,
    IronFist = 0x65,
    MotorDrive = 0x66,
    Unaware = 0x67,
    Rivalry = 0x68,
    BadDreams = 0x69,
    NoGuard = 0x6A,
    Normalize = 0x6B,
    SolidRock = 0x6C,
    QuickFeet = 0x6D,
    Filter = 0x6E,
    Klutz = 0x6F,
    Steadfast = 0x70,
    FlowerGift = 0x71,
    PoisonHeal = 0x72,
    MagicGuard = 0x73,
    Invalid = 0x74,
    HoneyGather = 0x75,
    Aftermath = 0x76,
    SnowCloak = 0x77,
    SnowWarning = 0x78,
    Forewarn = 0x79,
    StormDrain = 0x7A,
    LeafGuard = 0x7B,
    Unused0x7C = 0x7C,
    Unused0x7D = 0x7D,
    Unused0x7E = 0x7E,
    Unused0x7F = 0x7F,
    Unused0x80 = 0x80,
    Unused0x81 = 0x81,
    Unused0x82 = 0x82,
    Unused0x83 = 0x83,
    Unused0x84 = 0x84,
    Unused0x85 = 0x85,
    Unused0x86 = 0x86,
    Unused0x87 = 0x87,
    Unused0x88 = 0x88,
    Unused0x89 = 0x89,
    Unused0x8A = 0x8A,
    Unused0x8B = 0x8B,
    Unused0x8C = 0x8C,
    Unused0x8D = 0x8D,
    Unused0x8E = 0x8E,
    Unused0x8F = 0x8F,
    Unused0x90 = 0x90,
    Unused0x91 = 0x91,
    Unused0x92 = 0x92,
    Unused0x93 = 0x93,
    Unused0x94 = 0x94,
    Unused0x95 = 0x95,
    Unused0x96 = 0x96,
    Unused0x97 = 0x97,
    Unused0x98 = 0x98,
    Unused0x99 = 0x99,
    Unused0x9A = 0x9A,
    Unused0x9B = 0x9B,
    Unused0x9C = 0x9C,
    Unused0x9D = 0x9D,
    Unused0x9E = 0x9E,
    Unused0x9F = 0x9F,
    Unused0xA0 = 0xA0,
    Unused0xA1 = 0xA1,
    Unused0xA2 = 0xA2,
    Unused0xA3 = 0xA3,
    Unused0xA4 = 0xA4,
    Unused0xA5 = 0xA5,
    Unused0xA6 = 0xA6,
    Unused0xA7 = 0xA7,
    Unused0xA8 = 0xA8,
    Unused0xA9 = 0xA9,
    Unused0xAA = 0xAA,
    Unused0xAB = 0xAB,
    Unused0xAC = 0xAC,
    Unused0xAD = 0xAD,
    Unused0xAE = 0xAE,
    Unused0xAF = 0xAF,
    Unused0xB0 = 0xB0,
    Unused0xB1 = 0xB1,
    Unused0xB2 = 0xB2,
    Unused0xB3 = 0xB3,
    Unused0xB4 = 0xB4,
    Unused0xB5 = 0xB5,
    Unused0xB6 = 0xB6,
    Unused0xB7 = 0xB7,
    Unused0xB8 = 0xB8,
    Unused0xB9 = 0xB9,
    Unused0xBA = 0xBA,
    Unused0xBB = 0xBB,
    Unused0xBC = 0xBC,
    Unused0xBD = 0xBD,
    Unused0xBE = 0xBE,
    Unused0xBF = 0xBF,
    Unused0xC0 = 0xC0,
    Unused0xC1 = 0xC1,
    Unused0xC2 = 0xC2,
    Unused0xC3 = 0xC3,
    Unused0xC4 = 0xC4,
    Unused0xC5 = 0xC5,
    Unused0xC6 = 0xC6,
    Unused0xC7 = 0xC7,
    Unused0xC8 = 0xC8,
    Unused0xC9 = 0xC9,
    Unused0xCA = 0xCA,
    Unused0xCB = 0xCB,
    Unused0xCC = 0xCC,
    Unused0xCD = 0xCD,
    Unused0xCE = 0xCE,
    Unused0xCF = 0xCF,
    Unused0xD0 = 0xD0,
    Unused0xD1 = 0xD1,
    Unused0xD2 = 0xD2,
    Unused0xD3 = 0xD3,
    Unused0xD4 = 0xD4,
    Unused0xD5 = 0xD5,
    Unused0xD6 = 0xD6,
    Unused0xD7 = 0xD7,
    Unused0xD8 = 0xD8,
    Unused0xD9 = 0xD9,
    Unused0xDA = 0xDA,
    Unused0xDB = 0xDB,
    Unused0xDC = 0xDC,
    Unused0xDD = 0xDD,
    Unused0xDE = 0xDE,
    Unused0xDF = 0xDF,
    Unused0xE0 = 0xE0,
    Unused0xE1 = 0xE1,
    Unused0xE2 = 0xE2,
    Unused0xE3 = 0xE3,
    Unused0xE4 = 0xE4,
    Unused0xE5 = 0xE5,
    Unused0xE6 = 0xE6,
    Unused0xE7 = 0xE7,
    Unused0xE8 = 0xE8,
    Unused0xE9 = 0xE9,
    Unused0xEA = 0xEA,
    Unused0xEB = 0xEB,
    Unused0xEC = 0xEC,
    Unused0xED = 0xED,
    Unused0xEE = 0xEE,
    Unused0xEF = 0xEF,
    Unused0xF0 = 0xF0,
    Unused0xF1 = 0xF1,
    Unused0xF2 = 0xF2,
    Unused0xF3 = 0xF3,
    Unused0xF4 = 0xF4,
    Unused0xF5 = 0xF5,
    Unused0xF6 = 0xF6,
    Unused0xF7 = 0xF7,
    Unused0xF8 = 0xF8,
    Unused0xF9 = 0xF9,
    Unused0xFA = 0xFA,
    Unused0xFB = 0xFB,
    Unused0xFC = 0xFC,
    Unused0xFD = 0xFD,
    Unused0xFE = 0xFE,
    None = 0xFF,
    Null = 0x00,
}

#[derive(PrimitiveEnum_i8, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "python", derive(EnumToPy_i8))]
pub enum ShadowSize {
    Small = 0,
    Medium = 1,
    Large = 2,
}

#[derive(Clone, PackedStruct, Debug)]
#[packed_struct(endian = "lsb")]
pub struct MdEntryData {
    pub entid: u16,
    pub unk31: u16,
    pub national_pokedex_number: u16,
    pub base_movement_speed: u16,
    pub pre_evo_index: u16,
    #[packed_field(size_bytes = "2", ty = "enum")]
    pub evo_method: EvolutionMethod,
    pub evo_param1: u16,
    #[packed_field(size_bytes = "2", ty = "enum")]
    pub evo_param2: AdditionalRequirement,
    pub sprite_index: i16,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub gender: Gender,
    pub body_size: u8,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub type_primary: PokeType,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub type_secondary: PokeType,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub movement_type: MovementType,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub iq_group: IQGroup,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub ability_primary: Ability,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub ability_secondary: Ability,
    #[packed_field(size_bits = "1")]
    pub item_required_for_spawning: bool,
    #[packed_field(size_bits = "1")]
    pub can_evolve: bool,
    #[packed_field(size_bits = "1")]
    pub bitfield1_5: bool,
    #[packed_field(size_bits = "1")]
    pub can_move: bool,
    #[packed_field(size_bits = "1")]
    pub bitfield1_3: bool,
    #[packed_field(size_bits = "1")]
    pub bitfield1_2: bool,
    #[packed_field(size_bits = "1")]
    pub bitfield1_1: bool,
    #[packed_field(size_bits = "1")]
    pub bitfield1_0: bool,
    pub bitfield2: u8,
    pub exp_yield: u16,
    pub recruit_rate1: i16,
    pub base_hp: u16,
    pub recruit_rate2: i16,
    pub base_atk: u8,
    pub base_sp_atk: u8,
    pub base_def: u8,
    pub base_sp_def: u8,
    pub weight: i16,
    pub size: i16,
    pub unk17: u8,
    pub unk18: u8,
    #[packed_field(size_bytes = "1", ty = "enum")]
    pub shadow_size: ShadowSize,
    pub chance_spawn_asleep: i8,
    pub hp_regeneration: u8,
    pub unk21_h: i8,
    pub base_form_index: i16,
    pub exclusive_item1: i16,
    pub exclusive_item2: i16,
    pub exclusive_item3: i16,
    pub exclusive_item4: i16,
    pub unk27: i16,
    pub unk28: i16,
    pub unk29: i16,
    pub unk30: i16,
}

#[pyclass(module = "skytemple_rust.st_md")]
#[derive(Clone, Debug)]
pub struct MdEntry {
    pub data: MdEntryData,
    #[pyo3(get, set)]
    pub md_index: u32,
}

#[pymethods]
impl MdEntry {
    #[classmethod]
    pub fn new_empty(_cls: &PyType, entid: u16) -> Self {
        Self {
            md_index: 0,
            data: MdEntryData {
                entid,
                unk31: 0,
                national_pokedex_number: 0,
                base_movement_speed: 0,
                pre_evo_index: 0,
                evo_method: EvolutionMethod::None,
                evo_param1: 0,
                evo_param2: AdditionalRequirement::None,
                sprite_index: 0,
                gender: Gender::Invalid,
                body_size: 0,
                type_primary: PokeType::None,
                type_secondary: PokeType::None,
                movement_type: MovementType::Unknown1,
                iq_group: IQGroup::Invalid,
                ability_primary: Ability::None,
                ability_secondary: Ability::None,
                bitfield1_0: false,
                bitfield1_1: false,
                bitfield1_2: false,
                bitfield1_3: false,
                can_move: false,
                bitfield1_5: false,
                can_evolve: false,
                item_required_for_spawning: false,
                bitfield2: 0,
                exp_yield: 0,
                recruit_rate1: 0,
                base_hp: 0,
                recruit_rate2: 0,
                base_atk: 0,
                base_sp_atk: 0,
                base_def: 0,
                base_sp_def: 0,
                weight: 0,
                size: 0,
                unk17: 0,
                unk18: 0,
                shadow_size: ShadowSize::Small,
                chance_spawn_asleep: 0,
                hp_regeneration: 0,
                unk21_h: 0,
                base_form_index: 0,
                exclusive_item1: 0,
                exclusive_item2: 0,
                exclusive_item3: 0,
                exclusive_item4: 0,
                unk27: 0,
                unk28: 0,
                unk29: 0,
                unk30: 0,
            },
        }
    }

    #[getter]
    pub fn md_index_base(&self, py: Python) -> PyResult<u32> {
        Ok(self.md_index % MdPropertiesState::instance(py)?.borrow(py).num_entities)
    }

    // <editor-fold desc="Proxy getters for MdEntryData" defaultstate="collapsed">
    #[getter]
    #[cfg(feature = "python")]
    pub fn entid(&self) -> u16 {
        self.data.entid
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk31(&self) -> u16 {
        self.data.unk31
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn national_pokedex_number(&self) -> u16 {
        self.data.national_pokedex_number
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn base_movement_speed(&self) -> u16 {
        self.data.base_movement_speed
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn pre_evo_index(&self) -> u16 {
        self.data.pre_evo_index
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn evo_method(&self) -> EvolutionMethod {
        self.data.evo_method
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn evo_param1(&self) -> u16 {
        self.data.evo_param1
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn evo_param2(&self) -> AdditionalRequirement {
        self.data.evo_param2
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn sprite_index(&self) -> i16 {
        self.data.sprite_index
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn gender(&self) -> Gender {
        self.data.gender
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn body_size(&self) -> u8 {
        self.data.body_size
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn type_primary(&self) -> PokeType {
        self.data.type_primary
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn type_secondary(&self) -> PokeType {
        self.data.type_secondary
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn movement_type(&self) -> MovementType {
        self.data.movement_type
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn iq_group(&self) -> IQGroup {
        self.data.iq_group
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn ability_primary(&self) -> Ability {
        self.data.ability_primary
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn ability_secondary(&self) -> Ability {
        self.data.ability_secondary
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn bitfield1_0(&self) -> bool {
        self.data.bitfield1_0
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn bitfield1_1(&self) -> bool {
        self.data.bitfield1_1
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn bitfield1_2(&self) -> bool {
        self.data.bitfield1_2
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn bitfield1_3(&self) -> bool {
        self.data.bitfield1_3
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn can_move(&self) -> bool {
        self.data.can_move
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn bitfield1_5(&self) -> bool {
        self.data.bitfield1_5
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn can_evolve(&self) -> bool {
        self.data.can_evolve
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn item_required_for_spawning(&self) -> bool {
        self.data.item_required_for_spawning
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn bitfield2(&self) -> u8 {
        self.data.bitfield2
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn exp_yield(&self) -> u16 {
        self.data.exp_yield
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn recruit_rate1(&self) -> i16 {
        self.data.recruit_rate1
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn base_hp(&self) -> u16 {
        self.data.base_hp
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn recruit_rate2(&self) -> i16 {
        self.data.recruit_rate2
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn base_atk(&self) -> u8 {
        self.data.base_atk
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn base_sp_atk(&self) -> u8 {
        self.data.base_sp_atk
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn base_def(&self) -> u8 {
        self.data.base_def
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn base_sp_def(&self) -> u8 {
        self.data.base_sp_def
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn weight(&self) -> i16 {
        self.data.weight
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn size(&self) -> i16 {
        self.data.size
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk17(&self) -> u8 {
        self.data.unk17
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk18(&self) -> u8 {
        self.data.unk18
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn shadow_size(&self) -> ShadowSize {
        self.data.shadow_size
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn chance_spawn_asleep(&self) -> i8 {
        self.data.chance_spawn_asleep
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn hp_regeneration(&self) -> u8 {
        self.data.hp_regeneration
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk21_h(&self) -> i8 {
        self.data.unk21_h
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn base_form_index(&self) -> i16 {
        self.data.base_form_index
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn exclusive_item1(&self) -> i16 {
        self.data.exclusive_item1
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn exclusive_item2(&self) -> i16 {
        self.data.exclusive_item2
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn exclusive_item3(&self) -> i16 {
        self.data.exclusive_item3
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn exclusive_item4(&self) -> i16 {
        self.data.exclusive_item4
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk27(&self) -> i16 {
        self.data.unk27
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk28(&self) -> i16 {
        self.data.unk28
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk29(&self) -> i16 {
        self.data.unk29
    }

    #[getter]
    #[cfg(feature = "python")]
    pub fn unk30(&self) -> i16 {
        self.data.unk30
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_entid(&mut self, value: u16) {
        self.data.entid = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk31(&mut self, value: u16) {
        self.data.unk31 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_national_pokedex_number(&mut self, value: u16) {
        self.data.national_pokedex_number = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_base_movement_speed(&mut self, value: u16) {
        self.data.base_movement_speed = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_pre_evo_index(&mut self, value: u16) {
        self.data.pre_evo_index = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_evo_method(&mut self, value: EvolutionMethod) {
        self.data.evo_method = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_evo_param1(&mut self, value: u16) {
        self.data.evo_param1 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_evo_param2(&mut self, value: AdditionalRequirement) {
        self.data.evo_param2 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_sprite_index(&mut self, value: i16) {
        self.data.sprite_index = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_gender(&mut self, value: Gender) {
        self.data.gender = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_body_size(&mut self, value: u8) {
        self.data.body_size = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_type_primary(&mut self, value: PokeType) {
        self.data.type_primary = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_type_secondary(&mut self, value: PokeType) {
        self.data.type_secondary = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_movement_type(&mut self, value: MovementType) {
        self.data.movement_type = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_iq_group(&mut self, value: IQGroup) {
        self.data.iq_group = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_ability_primary(&mut self, value: Ability) {
        self.data.ability_primary = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_ability_secondary(&mut self, value: Ability) {
        self.data.ability_secondary = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_bitfield1_0(&mut self, value: bool) {
        self.data.bitfield1_0 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_bitfield1_1(&mut self, value: bool) {
        self.data.bitfield1_1 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_bitfield1_2(&mut self, value: bool) {
        self.data.bitfield1_2 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_bitfield1_3(&mut self, value: bool) {
        self.data.bitfield1_3 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_can_move(&mut self, value: bool) {
        self.data.can_move = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_bitfield1_5(&mut self, value: bool) {
        self.data.bitfield1_5 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_can_evolve(&mut self, value: bool) {
        self.data.can_evolve = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_item_required_for_spawning(&mut self, value: bool) {
        self.data.item_required_for_spawning = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_bitfield2(&mut self, value: u8) {
        self.data.bitfield2 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_exp_yield(&mut self, value: u16) {
        self.data.exp_yield = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_recruit_rate1(&mut self, value: i16) {
        self.data.recruit_rate1 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_base_hp(&mut self, value: u16) {
        self.data.base_hp = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_recruit_rate2(&mut self, value: i16) {
        self.data.recruit_rate2 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_base_atk(&mut self, value: u8) {
        self.data.base_atk = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_base_sp_atk(&mut self, value: u8) {
        self.data.base_sp_atk = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_base_def(&mut self, value: u8) {
        self.data.base_def = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_base_sp_def(&mut self, value: u8) {
        self.data.base_sp_def = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_weight(&mut self, value: i16) {
        self.data.weight = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_size(&mut self, value: i16) {
        self.data.size = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk17(&mut self, value: u8) {
        self.data.unk17 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk18(&mut self, value: u8) {
        self.data.unk18 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_shadow_size(&mut self, value: ShadowSize) {
        self.data.shadow_size = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_chance_spawn_asleep(&mut self, value: i8) {
        self.data.chance_spawn_asleep = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_hp_regeneration(&mut self, value: u8) {
        self.data.hp_regeneration = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk21_h(&mut self, value: i8) {
        self.data.unk21_h = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_base_form_index(&mut self, value: i16) {
        self.data.base_form_index = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_exclusive_item1(&mut self, value: i16) {
        self.data.exclusive_item1 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_exclusive_item2(&mut self, value: i16) {
        self.data.exclusive_item2 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_exclusive_item3(&mut self, value: i16) {
        self.data.exclusive_item3 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_exclusive_item4(&mut self, value: i16) {
        self.data.exclusive_item4 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk27(&mut self, value: i16) {
        self.data.unk27 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk28(&mut self, value: i16) {
        self.data.unk28 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk29(&mut self, value: i16) {
        self.data.unk29 = value;
    }

    #[setter]
    #[cfg(feature = "python")]
    pub fn set_unk30(&mut self, value: i16) {
        self.data.unk30 = value;
    }
    // </editor-fold>

    pub fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

#[pyclass(module = "skytemple_rust.st_md")]
pub struct MdIterator {
    iter: vec::IntoIter<Py<MdEntry>>,
}

impl MdIterator {
    pub fn new(iter: vec::IntoIter<Py<MdEntry>>) -> Self {
        Self { iter }
    }
}

#[pymethods]
impl MdIterator {
    pub fn __next__(&mut self) -> Option<Py<MdEntry>> {
        self.iter.next()
    }
}

#[pyclass(module = "skytemple_rust.st_md")]
#[derive(Clone, Debug)]
pub struct Md {
    #[pyo3(get, set)]
    pub entries: Vec<Py<MdEntry>>,
    cache_entries_by_entid: RefCell<BTreeMap<usize, Vec<Py<MdEntry>>>>,
}

#[pymethods]
impl Md {
    #[new]
    pub fn new(mut data: StBytes, py: Python) -> PyResult<Self> {
        static_assert_size!(<MdEntryData as PackedStruct>::ByteArray, 0x44);

        // Skip header:
        data.get_u32_le();
        let number_entries = data.get_u32_le();

        let slf = Self {
            entries: data
                .chunks_exact(size_of::<<MdEntryData as PackedStruct>::ByteArray>())
                .enumerate()
                .map(|(i, b)| {
                    <MdEntryData as PackedStruct>::unpack(b.try_into().unwrap()).map(|data| {
                        MdEntry {
                            md_index: i.try_into().unwrap(),
                            data,
                        }
                    })
                })
                .map(|mde| {
                    mde.map_err(convert_packing_err)
                        .and_then(|mde| Py::new(py, mde))
                })
                .collect::<PyResult<Vec<Py<MdEntry>>>>()?,
            cache_entries_by_entid: RefCell::new(BTreeMap::new()),
        };

        pyr_assert!(
            slf.entries.len() == number_entries as usize,
            "The amount of data in the Md file did not match it's header."
        );

        Ok(slf)
    }

    pub fn get_by_index(&self, index: usize) -> PyResult<Py<MdEntry>> {
        self.entries
            .get(index)
            .ok_or_else(|| exceptions::PyIndexError::new_err("Index for Md out of range."))
            .map(Clone::clone)
    }

    pub fn get_by_entity_id(&self, index: usize, py: Python) -> PyResult<Vec<(u32, Py<MdEntry>)>> {
        let mut cache_mut = self.cache_entries_by_entid.borrow_mut();
        match cache_mut.entry(index) {
            Entry::Vacant(ve) => {
                let new_list = self
                    .entries
                    .iter()
                    .filter(|e| e.borrow(py).data.entid as usize == index)
                    .cloned()
                    .collect();
                let new_list_ref = ve.insert(new_list);
                Self::get_by_entity_id_process(new_list_ref, py)
            }
            Entry::Occupied(oe) => Self::get_by_entity_id_process(oe.get(), py),
        }
    }

    #[cfg(feature = "python")]
    pub fn __len__(&self) -> usize {
        self.entries.len()
    }

    #[cfg(feature = "python")]
    pub fn __getitem__(&self, key: usize) -> PyResult<Py<MdEntry>> {
        self.get_by_index(key)
    }

    #[cfg(feature = "python")]
    pub fn __setitem__(&mut self, key: usize, value: Py<MdEntry>) -> PyResult<()> {
        let entry = self.entries.get_mut(key);
        if let Some(e) = entry {
            *e = value;
            Ok(())
        } else {
            Err(exceptions::PyIndexError::new_err(
                "Index for Md out of range.",
            ))
        }
    }

    #[cfg(feature = "python")]
    pub fn __delitem__(&mut self, key: usize) -> PyResult<()> {
        if key >= self.entries.len() {
            Err(exceptions::PyIndexError::new_err(
                "Index for Md out of range.",
            ))
        } else {
            self.entries.remove(key);
            Ok(())
        }
    }

    #[cfg(feature = "python")]
    pub fn __iter__(&mut self) -> MdIterator {
        MdIterator::new(self.entries.clone().into_iter())
    }
}

impl Md {
    fn get_by_entity_id_process(
        lst: &[Py<MdEntry>],
        py: Python,
    ) -> PyResult<Vec<(u32, Py<MdEntry>)>> {
        if lst.is_empty() {
            Err(exceptions::PyIndexError::new_err(
                "No entities with entid found.",
            ))
        } else {
            Ok(lst
                .iter()
                .map(|e| (e.borrow(py).md_index, e.clone()))
                .collect())
        }
    }
}

#[pyclass(module = "skytemple_rust.st_md")]
#[derive(Clone, Default)]
pub struct MdWriter;

#[pymethods]
impl MdWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<Md>, py: Python) -> PyResult<StBytes> {
        let mdl = model.borrow(py);

        let entries = mdl
            .entries
            .iter()
            .map(|entry| entry.borrow(py).data.pack().map_err(convert_packing_err))
            .collect::<PyResult<Vec<_>>>()?;

        let len_as_bytes = (mdl.entries.len() as u32).to_le_bytes();

        Ok(StBytes(
            b"MD\0\0"
                .iter()
                .chain(len_as_bytes.iter())
                .copied()
                .chain(entries.into_iter().flatten())
                .collect(),
        ))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_md_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_md";
    let m = PyModule::new(py, name)?;
    m.add_class::<MdPropertiesState>()?;
    m.add_class::<MdEntry>()?;
    m.add_class::<MdIterator>()?;
    m.add_class::<Md>()?;
    m.add_class::<MdWriter>()?;

    Ok((name, m))
}
