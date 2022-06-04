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
use crate::python::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, FromPrimitive)]
pub enum DmaType {
    Wall = 0,
    Water = 1,
    Floor = 2,
}

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for DmaType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let int: u8 = ob.extract()?;
        DmaType::from_u8(int).ok_or_else(|| {
            exceptions::PyValueError::new_err(format!("Invalid value {} for DmaType", int))
        })
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, FromPrimitive)]
pub enum DmaExtraType {
    Floor1 = 0,
    WallOrVoid = 1,
    Floor2 = 2,
}

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for DmaExtraType {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let int: u8 = ob.extract()?;
        DmaExtraType::from_u8(int).ok_or_else(|| {
            exceptions::PyValueError::new_err(format!("Invalid value {} for DmaExtraType", int))
        })
    }
}

pub const DMA_NEIGHBOR_SOUTH: u8 = 0x01;
pub const DMA_NEIGHBOR_SOUTH_EAST: u8 = 0x02;
pub const DMA_NEIGHBOR_EAST: u8 = 0x04;
pub const DMA_NEIGHBOR_NORTH_EAST: u8 = 0x08;
pub const DMA_NEIGHBOR_NORTH: u8 = 0x10;
pub const DMA_NEIGHBOR_NORTH_WEST: u8 = 0x20;
pub const DMA_NEIGHBOR_WEST: u8 = 0x40;
pub const DMA_NEIGHBOR_SOUTH_WEST: u8 = 0x80;

#[pyclass(module = "skytemple_rust.st_dma")]
#[derive(Clone)]
pub struct Dma {
    #[pyo3(get, set)]
    pub chunk_mappings: Vec<u8>,
}

#[pymethods]
impl Dma {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        Ok(Self {
            chunk_mappings: data.to_vec(),
        })
    }

    /// Returns all three variations (chunk ids) set for this dungeon tile configuration.
    /// neighbors_same is a bitfield with the bits for the directions set to 1 if the neighbor at this
    /// position has the same type as the tile at this position.
    /// TIP: For neighbors_same, use the bit flags of DMA_NEIGHBOR.
    pub fn get(&self, get_type: DmaType, neighbors_same: usize) -> Vec<u8> {
        let high_two = match get_type {
            DmaType::Wall => 0,
            DmaType::Water => 0x100,
            DmaType::Floor => 0x200,
        };
        let idx = high_two + neighbors_same;
        self.chunk_mappings[(idx * 3)..(idx * 3) + 3].to_vec()
    }

    /// Returns a few extra chunk variations for the given type.
    /// How they are used exactly by the game is currently not know,
    /// this interface could change if we find out.
    #[allow(unused_variables)]
    pub fn get_extra(&self, extra_type: DmaExtraType) -> Vec<u8> {
        //      cms = []
        //         for i in range(0x300 * 3, len(self.chunk_mappings)):
        //             if i % 3 == extra_type:
        //                 cms.append(self.chunk_mappings[i])
        //         return cms
        todo!()
    }

    /// Sets the mapping for the given configuration and the given variation of it.
    pub fn set(
        &mut self,
        get_type: DmaType,
        neighbors_same: usize,
        variation_index: usize,
        value: u8,
    ) {
        let high_two = match get_type {
            DmaType::Wall => 0,
            DmaType::Water => 0x100,
            DmaType::Floor => 0x200,
        };
        let idx = high_two + neighbors_same;
        self.chunk_mappings[(idx * 3) + variation_index] = value;
    }

    /// Sets and extra tile entry.
    #[allow(unused_variables)]
    pub fn set_extra(&mut self, extra_type: DmaExtraType, index: u8, value: usize) {
        //  self.chunk_mappings[(0x300 * 3) + extra_type + (3 * index)] = value
        todo!()
    }
}

impl Dma {
    // def get_tile_neighbors(wall_matrix: List[List[Union[int, bool]]], x, y, self_is_wall_or_water: bool,
    //                        treat_outside_as_wall=False):
    //     """Return the neighbor bit map for the given 3x3 matrix.
    //     1 means there is a wall / water. Out of bounds is read as floor, unless treat_outside_as_wall,
    //     then it's water/wall."""
    //     ns = 0
    //     if treat_outside_as_wall:
    //         # we enlarge the matrix and add a 1 chunk-sized border
    //         x += 1
    //         y += 1
    //         wall_matrix = [[1] * (len(wall_matrix[0]) + 2)] + \
    //                       [[1] + l + [1] for l in wall_matrix] + \
    //                       [[1] * (len(wall_matrix[0]) + 2)]
    //     # SOUTH
    //     if y + 1 < len(wall_matrix) and wall_matrix[y + 1][x]:
    //         ns += DmaNeighbor.SOUTH
    //     # SOUTH_EAST
    //     if y + 1 < len(wall_matrix) and x + 1 < len(wall_matrix[y + 1]) and wall_matrix[y + 1][x + 1]:
    //         ns += DmaNeighbor.SOUTH_EAST
    //     # EAST
    //     if x + 1 < len(wall_matrix[y]) and wall_matrix[y][x + 1]:
    //         ns += DmaNeighbor.EAST
    //     # NORTH_EAST
    //     if y - 1 >= 0 and x + 1 < len(wall_matrix[y - 1]) and wall_matrix[y - 1][x + 1]:
    //         ns += DmaNeighbor.NORTH_EAST
    //     # NORTH
    //     if y - 1 >= 0 and wall_matrix[y - 1][x]:
    //         ns += DmaNeighbor.NORTH
    //     # NORTH_WEST
    //     if y - 1 >= 0 and x - 1 >= 0 and wall_matrix[y - 1][x - 1]:
    //         ns += DmaNeighbor.NORTH_WEST
    //     # WEST
    //     if x - 1 >= 0 and wall_matrix[y][x - 1]:
    //         ns += DmaNeighbor.WEST
    //     # SOUTH_WEST
    //     if y + 1 < len(wall_matrix) and x - 1 >= 0 and wall_matrix[y + 1][x - 1]:
    //         ns += DmaNeighbor.SOUTH_WEST
    //
    //     if not self_is_wall_or_water:
    //         # If we are not solid, we need to invert, since we just checked for us being solid.
    //         ns ^= 0xFF
    //     return ns
}

#[pyclass(module = "skytemple_rust.st_dma")]
#[derive(Clone, Default)]
pub struct DmaWriter;

#[pymethods]
impl DmaWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<Dma>, py: Python) -> PyResult<StBytes> {
        Ok(StBytes::from(model.borrow(py).chunk_mappings.clone()))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_dma_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_dma";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dma>()?;
    m.add_class::<DmaWriter>()?;

    Ok((name, m))
}
