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
use crate::image::tilemap_entry::TilemapEntry;
use crate::image::{In256ColIndexedImage, IndexedImage};
use crate::python::*;
use crate::st_dpci::input::InputDpci;

#[pyclass(module = "skytemple_rust.st_dpc")]
#[derive(Clone)]
pub struct Dpc {
    #[pyo3(get, set)]
    pub chunks: Vec<Vec<TilemapEntry>>,
}

#[pymethods]
impl Dpc {
    #[allow(unused_variables)]
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        todo!()
    }

    #[allow(unused_variables)]
    #[args(width_in_mtiles = "16")]
    /// Convert all chunks of the DPC to one big image.
    /// The chunks are all placed next to each other.
    /// The resulting image has one large palette with all palettes merged together.
    ///
    /// To be used with the DPCI file for this dungeon.
    /// To get the palettes, use the data from the DPL file for this dungeon.
    pub fn chunks_to_pil(
        &self,
        dpci: InputDpci,
        palettes: Vec<Vec<u8>>,
        width_in_mtiles: usize,
    ) -> PyResult<IndexedImage> {
        todo!()
    }

    #[allow(unused_variables)]
    /// Convert a single chunk of the DPC into a image. For general notes, see chunks_to_pil.
    pub fn single_chunk_to_pil(
        &self,
        chunk_idx: usize,
        dpci: InputDpci,
        palettes: Vec<Vec<u8>>,
    ) -> PyResult<IndexedImage> {
        todo!()
    }

    #[args(force_import = "true")]
    #[allow(unused_variables)]
    /// Imports chunks. Format same as for chunks_to_pil.
    /// Replaces tile mappings and returns the new tiles for storing them in a DPCI and the palettes
    /// for storing in a DPL.
    //
    /// The image must have a palette containing the 16 sub-palettes with 16 colors each (256 colors).
    ///
    /// If a pixel in a tile uses a color outside of it's 16 color range the color is replaced with
    /// 0 of the palette (transparent). The "force_import" parameter is ignored.
    pub fn pil_to_chunks(
        &mut self,
        img: In256ColIndexedImage,
        force_import: bool,
    ) -> PyResult<(StBytes, Vec<Vec<u8>>)> {
        todo!()
    }

    #[allow(unused_variables)]
    #[args(contains_null_chunk = "false", correct_tile_ids = "true")]
    /// Replace the tile mappings of the specified layer.
    /// If contains_null_tile is False, the null chunk is added to the list, at the beginning.
    //
    /// If correct_tile_ids is True, then the tile id of tile_mappings is also increased by one. Use this,  TODO
    /// if you previously used import_tiles with contains_null_tile=False  TODO
    pub fn import_tile_mappings(
        &mut self,
        tile_mappings: Vec<Vec<TilemapEntry>>,
        contains_null_chunk: bool,
        correct_tile_ids: bool,
    ) {
        todo!()
    }
}

#[pyclass(module = "skytemple_rust.st_dpc")]
#[derive(Clone, Default)]
pub struct DpcWriter;

#[pymethods]
impl DpcWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    #[allow(unused_variables)]
    pub fn write(&self, model: Py<Dpc>, py: Python) -> PyResult<StBytes> {
        todo!()
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_dpc_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_dpc";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dpc>()?;
    m.add_class::<DpcWriter>()?;

    Ok((name, m))
}

/////////////////////////
/////////////////////////
// DPCs as inputs (for compatibility of including other DPC implementations from Python)
#[cfg(feature = "python")]
pub mod input {
    use crate::python::*;
    use crate::st_dpc::Dpc;

    pub trait DpcProvider: ToPyObject {}

    impl DpcProvider for Py<Dpc> {}

    impl DpcProvider for PyObject {}

    pub struct InputDpc(pub Box<dyn DpcProvider>);

    impl<'source> FromPyObject<'source> for InputDpc {
        fn extract(ob: &'source PyAny) -> PyResult<Self> {
            if let Ok(obj) = ob.extract::<Py<Dpc>>() {
                Ok(Self(Box::new(obj)))
            } else {
                Ok(Self(Box::new(ob.to_object(ob.py()))))
            }
        }
    }

    impl IntoPy<PyObject> for InputDpc {
        fn into_py(self, py: Python) -> PyObject {
            self.0.to_object(py)
        }
    }

    impl From<InputDpc> for Dpc {
        fn from(obj: InputDpc) -> Self {
            Python::with_gil(|py| obj.0.to_object(py).extract(py).unwrap())
        }
    }
}

#[cfg(not(feature = "python"))]
pub mod input {
    use crate::st_dpc::Dpc;

    pub trait DpcProvider {}

    impl DpcProvider for Dpc {}

    pub struct InputDpc(pub(crate) Dpc);

    impl From<InputDpc> for Dpc {
        fn from(obj: InputDpc) -> Self {
            obj.0
        }
    }
}
