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

use crate::bytes::{AsStBytes, StBytes};
use crate::gettext::gettext;
use crate::python::*;
use crate::st_mappa_bin::MappaBin;
use crate::st_sir0::{Sir0Error, Sir0Result, Sir0Serializable};
use crate::util::pad;
use anyhow::anyhow;
use bytes::{BufMut, Bytes, BytesMut};
use packed_struct::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::iter::once;
use std::num::TryFromIntError;

const EMPTY_MINIMIZED_FLOOR: [u8; 18] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static DEFAULT_FLOOR: MinimizedMappaFloor = MinimizedMappaFloor {
    layout_idx: 0,
    monsters_idx: 0,
    traps_idx: 0,
    floor_items_idx: 0,
    shop_items_idx: 0,
    monster_house_items_idx: 0,
    buried_items_idx: 0,
    unk_items1_idx: 0,
    unk_items2_idx: 0,
};

enum FloorMinimized<'a> {
    Default,
    Floor(&'a MinimizedMappaFloor),
}

#[derive(Default, PackedStruct)]
#[packed_struct(endian = "lsb")]
pub struct MinimizedMappaFloor {
    layout_idx: u16,
    monsters_idx: u16,
    traps_idx: u16,
    floor_items_idx: u16,
    shop_items_idx: u16,
    monster_house_items_idx: u16,
    buried_items_idx: u16,
    unk_items1_idx: u16,
    unk_items2_idx: u16,
}

pub struct MinimizedMappa {
    pub floor_lists: Vec<Vec<MinimizedMappaFloor>>,
    pub layouts: Vec<Bytes>,
    pub monsters: Vec<Bytes>,
    pub traps: Vec<Bytes>,
    pub items: Vec<Bytes>,
}

impl MinimizedMappa {
    pub fn from_mappa(mappa: &MappaBin) -> Self {
        let mut o_floor_lists = Vec::with_capacity(mappa.floor_lists.len());
        let mut o_layout = Vec::with_capacity(1000);
        let mut o_monsters = Vec::with_capacity(1000);
        let mut o_traps = Vec::with_capacity(1000);
        let mut o_items = Vec::with_capacity(4000);
        let mut h_layout = Vec::with_capacity(1000);
        let mut h_monsters = Vec::with_capacity(1000);
        let mut h_traps = Vec::with_capacity(1000);
        let mut h_items = Vec::with_capacity(4000);

        Python::with_gil(|py| {
            for floor_list in &mappa.floor_lists {
                let mut o_floor_list = Vec::with_capacity(floor_list.len());

                for floor in floor_list {
                    let floor_brw = floor.borrow(py);
                    debug_assert_eq!(32, floor_brw.layout.as_bytes().len());
                    o_floor_list.push(MinimizedMappaFloor {
                        layout_idx: Self::find_or_insert(
                            &mut o_layout,
                            &mut h_layout,
                            &floor_brw.layout,
                        ),
                        monsters_idx: Self::find_or_insert(
                            &mut o_monsters,
                            &mut h_monsters,
                            &floor_brw.monsters,
                        ),
                        traps_idx: Self::find_or_insert(
                            &mut o_traps,
                            &mut h_traps,
                            &floor_brw.traps,
                        ),
                        floor_items_idx: Self::find_or_insert(
                            &mut o_items,
                            &mut h_items,
                            &floor_brw.floor_items,
                        ),
                        shop_items_idx: Self::find_or_insert(
                            &mut o_items,
                            &mut h_items,
                            &floor_brw.shop_items,
                        ),
                        monster_house_items_idx: Self::find_or_insert(
                            &mut o_items,
                            &mut h_items,
                            &floor_brw.monster_house_items,
                        ),
                        buried_items_idx: Self::find_or_insert(
                            &mut o_items,
                            &mut h_items,
                            &floor_brw.buried_items,
                        ),
                        unk_items1_idx: Self::find_or_insert(
                            &mut o_items,
                            &mut h_items,
                            &floor_brw.unk_items1,
                        ),
                        unk_items2_idx: Self::find_or_insert(
                            &mut o_items,
                            &mut h_items,
                            &floor_brw.unk_items2,
                        ),
                    })
                }

                o_floor_lists.push(o_floor_list);
            }
        });

        Self {
            floor_lists: o_floor_lists,
            layouts: o_layout,
            monsters: o_monsters,
            traps: o_traps,
            items: o_items,
        }
    }

    fn find_or_insert<T>(storage: &mut Vec<Bytes>, hash_storage: &mut Vec<u64>, source: T) -> u16
    where
        T: AsStBytes,
    {
        let source_bytes = source.as_bytes().0;
        let source_hash = Self::calculate_hash(&source_bytes);
        debug_assert_eq!(
            storage.contains(&source_bytes),
            hash_storage.contains(&source_hash)
        );
        let index = {
            if let Some(index) = hash_storage.iter().position(|&h| h == source_hash) {
                index
            } else {
                let index = hash_storage.len();
                hash_storage.push(source_hash);
                storage.push(source_bytes);
                index
            }
        };
        index.try_into().unwrap()
    }

    fn calculate_hash<T>(t: &T) -> u64
    where
        T: Hash,
    {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

impl Sir0Serializable for MinimizedMappa {
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<u32>, Option<u32>)> {
        static_assert_size!(<MinimizedMappaFloor as PackedStruct>::ByteArray, 18);

        let mut pointer_offsets: Vec<u32> = Vec::with_capacity(self.floor_lists.len() * 200);

        let mut data = self
            .floor_lists
            .iter()
            // The default floor at the beginning is the null floor at the start of all floor lists.
            .flat_map(|fl| {
                once(FloorMinimized::Default)
                    .chain(fl.iter().map(FloorMinimized::Floor))
            })
            .map(|f| {
                match f {
                    FloorMinimized::Default => {
                        Ok(DEFAULT_FLOOR.pack().unwrap())
                    }
                    FloorMinimized::Floor(f) => {
                        let byf = f.pack().unwrap();
                        if byf == EMPTY_MINIMIZED_FLOOR {
                            Err(exceptions::PyValueError::new_err(gettext(
                                "Could not save floor: It contains too much empty data.\nThis probably happened because a lot of spawn lists are empty.\nPlease check the floors you edited and fill them with more data. If you are using the randomizer, check your allowed item list."
                            )))
                        } else {
                            Ok(byf)
                        }
                    }
                }
            })
            .collect::<PyResult<Vec<_>>>()
            .map_err(Sir0Error::SerializeFailedPy)?
            .into_iter()
            .flatten()
            .collect::<BytesMut>();

        // Padding
        pad(&mut data, 16, 0);

        // Floor list LUT
        let start_floor_list_lut = data.len() as u32;
        let mut cursor_floor_data: u32 = 0;
        data.reserve(self.floor_lists.len() * 4);
        for fl in &self.floor_lists {
            pointer_offsets.push(data.len() as u32);
            data.put_u32_le(cursor_floor_data);
            cursor_floor_data = cursor_floor_data
                .checked_add(
                    ((fl.len() + 1) * 18)
                        .try_into()
                        .map_err(convert_try_from_int)?,
                )
                .ok_or_else(|| {
                    Sir0Error::SerializeFailed(anyhow!("Floor list too big to write."))
                })?;
        }

        // Padding
        pad(&mut data, 4, 0xAA);

        // Floor layout data
        let start_floor_layout_data = data.len() as u32;
        data.extend(self.layouts.iter().flatten());

        // Padding
        pad(&mut data, 4, 0xAA);

        // Monster spawn data
        let mut monster_data_pointer = Vec::with_capacity(self.monsters.len());
        for monster_list in &self.monsters {
            monster_data_pointer.push(data.len() as u32);
            data.extend_from_slice(&monster_list[..]);
        }

        // Padding
        pad(&mut data, 4, 0xAA);

        // Monster spawn LUT
        let start_monster_lut = data.len() as u32;
        data.reserve(4 * monster_data_pointer.len());
        for pnt in monster_data_pointer {
            pointer_offsets.push(data.len() as u32);
            data.put_u32_le(pnt);
        }

        // Padding
        pad(&mut data, 4, 0xAA);

        // Trap lists data
        let mut trap_data_pointer = Vec::with_capacity(self.traps.len());
        for trap_list in &self.traps {
            trap_data_pointer.push(data.len() as u32);
            data.extend_from_slice(&trap_list[..]);
        }

        // Padding
        pad(&mut data, 16, 0xAA);

        // Trap lists LUT
        let start_traps_lut = data.len() as u32;
        data.reserve(4 * trap_data_pointer.len());
        for pnt in trap_data_pointer {
            pointer_offsets.push(data.len() as u32);
            data.put_u32_le(pnt);
        }

        // Item spawn lists data
        let mut item_data_pointer = Vec::with_capacity(self.items.len());
        for item_list in &self.items {
            item_data_pointer.push(data.len() as u32);
            data.extend_from_slice(&item_list[..]);
        }

        // Padding
        pad(&mut data, 16, 0xAA);

        // Item spawn lists LUT
        let start_items_lut = data.len() as u32;
        data.reserve(4 * item_data_pointer.len());
        for pnt in item_data_pointer {
            pointer_offsets.push(data.len() as u32);
            data.put_u32_le(pnt);
        }

        // Padding
        pad(&mut data, 16, 0xAA);

        let data_pointer: u32 = data.len() as u32;
        data.reserve(4 * 5);
        pointer_offsets.push(data.len() as u32);
        data.put_u32_le(start_floor_list_lut);
        pointer_offsets.push(data.len() as u32);
        data.put_u32_le(start_floor_layout_data);
        pointer_offsets.push(data.len() as u32);
        data.put_u32_le(start_items_lut);
        pointer_offsets.push(data.len() as u32);
        data.put_u32_le(start_monster_lut);
        pointer_offsets.push(data.len() as u32);
        data.put_u32_le(start_traps_lut);

        // Check if data.len() fits into u32. If it doesn't, one of the pointer offsets above
        // and before that will have overflown...
        u32::try_from(data.len()).map_err(convert_try_from_int)?;

        Ok((data.into(), pointer_offsets, Some(data_pointer)))
    }

    fn sir0_unwrap(content_data: StBytes, data_pointer: u32) -> Sir0Result<Self> {
        MappaBin::sir0_unwrap(content_data, data_pointer).map(|m| Self::from_mappa(&m))
    }
}

fn convert_try_from_int(e: TryFromIntError) -> Sir0Error {
    Sir0Error::SerializeFailed(e.into())
}
