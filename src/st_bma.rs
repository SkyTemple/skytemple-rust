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
use crate::image::{In256ColIndexedImage, IndexedImage};
use crate::python::*;
use crate::st_bpa::input::InputBpa;
use crate::st_bpc::input::InputBpc;
use crate::st_bpl::input::InputBpl;

#[pyclass(module = "skytemple_rust.st_bma")]
#[derive(Clone)]
pub struct Bma {
    #[pyo3(get, set)]
    pub map_width_camera: u16,
    #[pyo3(get, set)]
    pub map_height_camera: u16,
    #[pyo3(get, set)]
    pub tiling_width: u8,
    #[pyo3(get, set)]
    pub tiling_height: u8,
    #[pyo3(get, set)]
    pub map_width_chunks: u16,
    #[pyo3(get, set)]
    pub map_height_chunks: u16,
    #[pyo3(get, set)]
    pub number_of_layers: u8,
    #[pyo3(get, set)]
    pub unk6: u8,
    #[pyo3(get, set)]
    pub number_of_collision_layers: u8,

    #[pyo3(get, set)]
    pub layer0: Vec<u8>,
    #[pyo3(get, set)]
    pub layer1: Option<Vec<u8>>,

    // if unk6:
    #[pyo3(get, set)]
    pub unknown_data_block: Option<Vec<u8>>,
    // if number_of_collision_layers > 0:
    #[pyo3(get, set)]
    pub collision: Option<Vec<bool>>,
    // if number_of_collision_layers > 1:
    #[pyo3(get, set)]
    pub collision2: Option<Vec<bool>>
}

#[pymethods]
impl Bma {
    #[new]
    pub fn new(data: StBytes, py: Python) -> PyResult<Self> {
        //         from skytemple_files.common.types.file_types import FileType
        //         if not isinstance(data, memoryview):
        //             data = memoryview(data)
        //
        //         self.map_width_camera = read_uintle(data, 0)
        //         self.map_height_camera = read_uintle(data, 1)
        //         # ALL game maps have the same values here. Changing them does nothing,
        //         # so the game seems to be hardcoded to 3x3.
        //         self.tiling_width = read_uintle(data, 2)
        //         self.tiling_height = read_uintle(data, 3)
        //         # Map width & height in chunks, so map.map_width_camera / map.tiling_width
        //         # The only maps this is not true for are G01P08A. S01P01B, S15P05A, S15P05B, it seems they
        //         # are missing one tile in width (32x instead of 33x)
        //         # The game doesn't seem to care if this value is off by less than 3 (tiling_w/h).
        //         # But NOTE that this has consequences for the collision and unknown data layers! See notes at collision
        //         # below!
        //         self.map_width_chunks = read_uintle(data, 4)
        //         self.map_height_chunks = read_uintle(data, 5)
        //         # Through tests against the BPC, it was determined that unk5 is the number of layers:
        //         # It seems to be ignored by the game, however
        //         self.number_of_layers = read_uintle(data, 6, 2)
        //         # Some kind of boolean flag? Seems to control if there is a third data block between
        //         # layer data and collision - Seems to be related to NPC conversations, see below.
        //         self.unk6 = read_uintle(data, 8, 2)
        //         # Some maps weirdly have 0x02 here and then have two collision layers, but they always seem redundant?
        //         self.number_of_collision_layers = read_uintle(data, 0xA, 2)
        //
        //         # in p01p01a: 0xc - 0x27: Layer 1 header? 0xc messes everthing up. after that each row? 27 rows...?
        //         #             0xc -> 0xc8 = 200
        //         #             Same again? 0x28 is 0xc8 again. 0x29 - 0x43 are 27 rows for layer 1 again... Seems to repeat, with some odd ones in between
        //         #             Sometimes 0xC6 instead: Only 21 entries?
        //         #             -> UNTIL 0x214. At 0x215 Layer 2 starts.
        //         #             SEEMS TO BE NRL ENCODING:
        //         #                   2x12 bits of information stored in 3 bytes. C8 ->
        //         #                       Copy next and repeat 8 types (=9). 3x9=27!
        //         #               Decompressed length = (map_width_chunks*map_height_chunks*12)/8
        //         #                   = 513
        //         #               27 / 1,5 = 18! So the first set contains each tile for each row.
        //         #
        //         #               This seems to be the solution, it's the same for RRT
        //         #               [It also explains why changing a tile changes all tiles below!!]:
        //         #               "Each row has a series of Pair-24 NRL compressed values,
        //         #               one for each chunk column (rounded up to the nearest even number).
        //         #               These values are xor'ed with the respective indices of the previous
        //         #               row to get the actual indices of the chunks in the current row.
        //         #               The first row assumes all previous indices were 0."
        //         #               source:
        //         #               https://projectpokemon.org/docs/mystery-dungeon-nds/rrt-background-format-r113/
        //         #
        //         number_of_bytes_per_layer = self.map_width_chunks * self.map_height_chunks * 2
        //         # If the map width is odd, we have one extra tile per row:
        //         if self.map_width_chunks % 2 != 0:
        //             number_of_bytes_per_layer += self.map_height_chunks * 2
        //         number_of_bytes_per_layer = math.ceil(number_of_bytes_per_layer)
        //
        //         # Read first layer
        //         #print(f"r> layer 0x{0xC:02x}")
        //         self.layer0, compressed_layer0_size = self._read_layer(FileType.BMA_LAYER_NRL.decompress(
        //             data[0xC:],
        //             stop_when_size=number_of_bytes_per_layer
        //         ))
        //         self.layer1: Optional[List[int]] = None
        //         compressed_layer1_size = 0
        //         if self.number_of_layers > 1:
        //             # Read second layer
        //             #print(f"r> layer 0x{0xC + compressed_layer0_size:02x}")
        //             self.layer1, compressed_layer1_size = self._read_layer(FileType.BMA_LAYER_NRL.decompress(
        //                 data[0xC + compressed_layer0_size:],
        //                 stop_when_size=number_of_bytes_per_layer
        //             ))
        //
        //         offset_begin_next = 0xC + compressed_layer0_size + compressed_layer1_size
        //         self.unknown_data_block: Optional[List[int]] = None
        //         if self.unk6:
        //             # Unknown data block in generic NRL for "chat places"?
        //             # Seems to have something to do with counters? Like shop counters / NPC interactions.
        //             # Theory from looking at the maps:
        //             # It seems that if the player tries interact on these blocks, the game checks the other blocks for NPCs
        //             # to interact with (as if the player were standing on them)
        //             #print(f"r> unk   0x{offset_begin_next:02x}")
        //             self.unknown_data_block, data_block_len = self._read_unknown_data_block(FileType.GENERIC_NRL.decompress(
        //                 data[offset_begin_next:],
        //                 # It is unknown what size calculation is actually used here in game, see notes below for collision
        //                 # (search for 'NOTE!!!')
        //                 # We assume it's the same as for the collision.
        //                 stop_when_size=self.map_width_camera * self.map_height_camera
        //             ))
        //             offset_begin_next += data_block_len
        //         self.collision = None
        //         if self.number_of_collision_layers > 0:
        //             # Read level collision
        //             # The collision is stored like this:
        //             # RLE:
        //             # Each byte codes one byte, that can have the values 0 or 1.
        //             # The highest bit determines whether to output 0 or 1 bytes. All the other
        //             # bits form an unsigned integer. This int determines the number of bytes to output. 1 extra
        //             # byte is always output. (So 0x03 means output 4 0 bytes and 0x83 means output 4 1 bytes).
        //             # The maximum value of repeats is the map with in 8x8 tiles. So for a 54 tile map, the max values
        //             # are 0x53 and 0xC3.
        //             #
        //             # To get the actual collision value, all bytes are XORed with the value of the tile in the previous row,
        //             # this is the same principle as for the layer tile indices.
        //             # False (0) = Walktru; True (1) = Solid
        //             #
        //             # NOTE!!! Tests have shown, that the collision layers use map_width_camera and map_height_camera
        //             #         instead of map_width/height_chunks * tiling_width/height. The map that proves this is G01P08A!
        //             #print(f"r> col   0x{offset_begin_next:02x}")
        //             number_of_bytes_for_col = self.map_width_camera * self.map_height_camera
        //             self.collision, collision_size = self._read_collision(FileType.BMA_COLLISION_RLE.decompress(
        //                 data[offset_begin_next:],
        //                 stop_when_size=number_of_bytes_for_col
        //             ))
        //             offset_begin_next += collision_size
        //         self.collision2 = None
        //         if self.number_of_collision_layers > 1:
        //             # A second collision layer...?
        //             number_of_bytes_for_col = self.map_width_camera * self.map_height_camera
        //             self.collision2, collision_size2 = self._read_collision(FileType.BMA_COLLISION_RLE.decompress(
        //                 data[offset_begin_next:],
        //                 stop_when_size=number_of_bytes_for_col
        //             ))
        todo!()
    }
    pub fn to_pil_single_layer(&self, bpc: InputBpc, palettes: Vec<Vec<u8>>, bpas: Vec<Option<InputBpa>>, layer: u8) -> IndexedImage {
        todo!()
    }
    #[allow(clippy::too_many_arguments)]
    #[args(include_collision = "true", include_unknown_data_block = "true", pal_ani = "true", single_frame = "false")]
    pub fn to_pil(
        &self, bpc: InputBpc, bpl: InputBpl, bpas: Vec<Option<InputBpa>>, include_collision: bool,
        include_unknown_data_block: bool, pal_ani: bool, single_frame: bool
    ) -> Vec<IndexedImage> {
        todo!()
    }
    #[allow(clippy::too_many_arguments)]
    #[args(lower_img = "None", upper_img = "None", force_import = "true", how_many_palettes_lower_layer = "16")]
    pub fn from_pil(
        &mut self, bpc: InputBpc, bpl: InputBpl, lower_img: Option<In256ColIndexedImage>,
        upper_img: Option<In256ColIndexedImage>, force_import: bool,
        how_many_palettes_lower_layer: u16
    ) -> PyResult<()> {
        todo!()
    }
    pub fn remove_upper_layer(&self) -> PyResult<()> {
        todo!()
    }
    pub fn add_upper_layer(&self) -> PyResult<()> {
        todo!()
    }
    pub fn resize(&self, new_width_chunks: u16, new_height_chunks: u16, new_width_camera: u16, new_height_camera: u16) -> PyResult<()> {
        todo!()
    }
    pub fn place_chunk(&self, layer_id: u8, x: u16, y: u16, chunk_index: u16) -> PyResult<()> {
        todo!()
    }
}

impl Bma {
    fn read_layer(&self, data: ()) -> () {
        //     def _read_layer(self, data: Tuple[bytes, int]) -> Tuple[List[int], int]:
        //         # To get the actual index of a chunk, the value is XORed with the tile value right above!
        //         previous_row_values = [0 for _ in range(0, self.map_width_chunks)]
        //         layer: List[int] = []
        //         max_tiles = self.map_width_chunks * self.map_height_chunks
        //         i = 0
        //         skipped_on_prev = True
        //         for chunk in iter_bytes(data[0], 2):
        //             chunk_i = int.from_bytes(chunk, 'little')
        //             if i >= max_tiles:
        //                 # this happens if there is a leftover 12bit word.
        //                 break
        //             index_in_row = i % self.map_width_chunks
        //             # If the map width is odd, there is one extra chunk at the end of every row,
        //             # we remove this chunk.
        //             if not skipped_on_prev and index_in_row == 0 and self.map_width_chunks % 2 != 0:
        //                 skipped_on_prev = True
        //                 continue
        //             skipped_on_prev = False
        //             cv = chunk_i ^ previous_row_values[index_in_row]
        //             previous_row_values[index_in_row] = cv
        //             layer.append(cv)
        //             i += 1
        //         return layer, data[1]
        todo!()
    }

    fn read_collision(&self, data: ()) -> () {
        //     def _read_collision(self, data: Tuple[bytes, int]) -> Tuple[List[bool], int]:
        //         # To get the actual index of a chunk, the value is XORed with the tile value right above!
        //         previous_row_values = [False for _ in range(0, self.map_width_camera)]
        //         col = []
        //         for i, chunk in enumerate(data[0]):
        //             index_in_row = i % self.map_width_camera
        //             cv = bool(chunk ^ int(previous_row_values[index_in_row]))
        //             previous_row_values[index_in_row] = cv
        //             col.append(cv)
        //         return col, data[1]
        todo!()
    }

    fn read_unknown_data_block(&self, data: ()) -> () {
        //     def _read_unknown_data_block(data: Tuple[bytes, int]) -> Tuple[List[int], int]:
        //         # TODO: There doesn't seem to be this XOR thing here?
        //         unk = []
        //         for i, chunk in enumerate(data[0]):
        //             unk.append(chunk)
        //         return unk, data[1]
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_bma")]
#[derive(Clone, Default)]
pub struct BmaWriter;

#[pymethods]
impl BmaWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Py<Bma>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);
        //     def write(self, model: Bma) -> bytes:
        //         # First collect the layers, collision layers and unknown data layer,
        //         # so we know the sizes
        //         self.model = model
        //         layers = []
        //         collisions = []
        //         unknown_data = None
        //         size = 0xC  # Header
        //         for i in range(0, self.model.number_of_layers):
        //             layer = self._convert_layer(i)
        //             size += len(layer)
        //             layers.append(layer)
        //         for i in range(0, self.model.number_of_collision_layers):
        //             col = self._convert_collision(i)
        //             size += len(col)
        //             collisions.append(col)
        //         if self.model.unk6:
        //             unknown_data = self._convert_unknown_data_layer()
        //             size += len(unknown_data)
        //
        //         self.data = bytearray(size)
        //
        //         self._write_byte(self.model.map_width_camera)
        //         self._write_byte(self.model.map_height_camera)
        //         self._write_byte(self.model.tiling_width)
        //         self._write_byte(self.model.tiling_height)
        //         self._write_byte(self.model.map_width_chunks)
        //         self._write_byte(self.model.map_height_chunks)
        //         assert self.model.number_of_layers < 3
        //         self._write_16uintle(self.model.number_of_layers)
        //         assert self.model.unk6 < 2
        //         self._write_16uintle(self.model.unk6)
        //         assert self.model.number_of_collision_layers < 3
        //         self._write_16uintle(self.model.number_of_collision_layers)
        //
        //         self.bytes_written = 0xC
        //
        //         for layer in layers:
        //             lenlayer = len(layer)
        //             self.data[self.bytes_written:self.bytes_written+lenlayer] = layer
        //             #print(f"w> layer 0x{self.bytes_written:02x}")
        //             self.bytes_written += lenlayer
        //
        //         if unknown_data:
        //             lendata = len(unknown_data)
        //             self.data[self.bytes_written:self.bytes_written+lendata] = unknown_data
        //             #print(f"w> unk   0x{self.bytes_written:02x}")
        //             self.bytes_written += lendata
        //
        //         for col in collisions:
        //             lencol = len(col)
        //             self.data[self.bytes_written:self.bytes_written+lencol] = col
        //             #print(f"w> col   0x{self.bytes_written:02x}")
        //             self.bytes_written += lencol
        //
        //         return self.data
        todo!()
    }
}

impl BmaWriter {
    //
    //     def _convert_layer(self, layeri: int) -> bytes:
    //         """
    //         Converts chunk mappings for a layer back into bytes.
    //         If map size is odd, adds one extra tiles per row.
    //         Every row is NRL encoded separately, because the game decodes the rows separately!
    //         """
    //         from skytemple_files.common.types.file_types import FileType
    //
    //         layer = self.model.layer0 if layeri == 0 else self.model.layer1
    //         assert layer is not None
    //
    //         # The actual values are "encoded" using XOR.
    //         previous_row_values = [0 for _ in range(0, self.model.map_width_chunks)]
    //         size = self.model.map_width_chunks * self.model.map_height_chunks * 2
    //         assert size == len(layer) * 2
    //         if self.model.map_width_chunks % 2 != 0:
    //             # Keep in mind there's an extra null tile to be added per row
    //             size += self.model.map_height_chunks * 2
    //
    //         layer_bytes = bytearray(size)
    //         layer_bytes_cursor = 0
    //
    //         # Each tile is separately encoded, so we also build them separately
    //         for row in range(0, self.model.map_height_chunks):
    //             row_bytes = bytearray(int(size / self.model.map_height_chunks))
    //             for col in range(0, self.model.map_width_chunks):
    //                 i = row * self.model.map_width_chunks + col
    //                 actual_value = layer[i] ^ previous_row_values[col]
    //                 write_uintle(row_bytes, actual_value, col*2, 2)
    //                 previous_row_values[col] = layer[i]
    //             assert len(row_bytes) == int(size / self.model.map_height_chunks)
    //             # Extra null tile is already there because of the bytearray size!
    //             comp_row_bytes = FileType.BMA_LAYER_NRL.compress(row_bytes)
    //             len_comp_row_bytes = len(comp_row_bytes)
    //             layer_bytes[layer_bytes_cursor:layer_bytes_cursor+len_comp_row_bytes] = comp_row_bytes
    //             layer_bytes_cursor += len_comp_row_bytes
    //
    //         return layer_bytes[:layer_bytes_cursor]
    //
    //     def _convert_collision(self, layeri: int) -> bytes:
    //         """
    //         Converts collision mappings back into bytes.
    //         If map size is odd, adds one extra tiles per row
    //         Every row is NRL encoded separately, because the game decodes the rows separately!
    //         """
    //         from skytemple_files.common.types.file_types import FileType
    //
    //         collision_layer = self.model.collision if layeri == 0 else self.model.collision2
    //
    //         # The actual values are "encoded" using XOR.
    //         previous_row_values = [0 for _ in range(0, self.model.map_width_camera)]
    //         size = self.model.map_width_camera * self.model.map_height_camera
    //         assert size == len(collision_layer)  # type: ignore
    //
    //         layer_bytes = bytearray(size)
    //         layer_bytes_cursor = 0
    //
    //         # Each tile is separately encoded, so we also build them separately
    //         for row in range(0, self.model.map_height_camera):
    //             row_bytes = bytearray(int(size / self.model.map_height_camera))
    //             for col in range(0, self.model.map_width_camera):
    //                 i = row * self.model.map_width_camera + col
    //                 actual_value = collision_layer[i] ^ previous_row_values[col]  # type: ignore
    //                 write_uintle(row_bytes, actual_value, col)
    //                 previous_row_values[col] = collision_layer[i]  # type: ignore
    //             assert len(row_bytes) == int(size / self.model.map_height_camera)
    //             comp_row_bytes = FileType.BMA_COLLISION_RLE.compress(row_bytes)
    //             len_comp_row_bytes = len(comp_row_bytes)
    //             layer_bytes[layer_bytes_cursor:layer_bytes_cursor+len_comp_row_bytes] = comp_row_bytes
    //             layer_bytes_cursor += len_comp_row_bytes
    //
    //         return layer_bytes[:layer_bytes_cursor]
    //
    //     def _convert_unknown_data_layer(self) -> bytes:
    //         """
    //         Converts the unknown data layer back into bytes
    //         Every row is NRL encoded separately, because the game decodes the rows separately!
    //         """
    //         from skytemple_files.common.types.file_types import FileType
    //
    //         size = self.model.map_width_camera * self.model.map_height_camera
    //         assert self.model.unknown_data_block is not None
    //         assert size == len(self.model.unknown_data_block)
    //
    //         layer_bytes = bytearray(size)
    //         layer_bytes_cursor = 0
    //         # Each tile is separately encoded, so we also build them separately
    //         for row in range(0, self.model.map_height_camera):
    //             row_bytes = bytearray(int(size / self.model.map_height_camera))
    //             for col in range(0, self.model.map_width_camera):
    //                 i = row * self.model.map_width_camera + col
    //                 actual_value = self.model.unknown_data_block[i]
    //                 write_uintle(row_bytes, actual_value, col)
    //             assert len(row_bytes) == int(size / self.model.map_height_camera)
    //             comp_row_bytes = FileType.GENERIC_NRL.compress(row_bytes)
    //             len_comp_row_bytes = len(comp_row_bytes)
    //             layer_bytes[layer_bytes_cursor:layer_bytes_cursor+len_comp_row_bytes] = comp_row_bytes
    //             layer_bytes_cursor += len_comp_row_bytes
    //
    //         return layer_bytes[:layer_bytes_cursor]
    //
    //     def _write_16uintle(self, val: int) -> None:
    //         assert val <= 0xffff
    //         write_uintle(self.data, val, self.bytes_written, 2)
    //         self.bytes_written += 2
    //
    //     def _write_byte(self, val: int) -> None:
    //         assert val <= 0xff
    //         write_uintle(self.data, val, self.bytes_written)
    //         self.bytes_written += 1
}

#[cfg(feature = "python")]
pub(crate) fn create_st_bma_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_bma";
    let m = PyModule::new(py, name)?;
    m.add_class::<Bma>()?;
    m.add_class::<BmaWriter>()?;

    Ok((name, m))
}
