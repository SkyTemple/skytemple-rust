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
use crate::image::tiled::TiledImage;
use crate::image::tilemap_entry::TilemapEntry;
use crate::image::{In256ColIndexedImage, InIndexedImage, IndexedImage, PixelGenerator};
use crate::python::*;

pub const DPCI_TILE_DIM: usize = 8;

#[pyclass(module = "skytemple_rust.st_dpci")]
#[derive(Clone)]
pub struct Dpci {
    #[pyo3(get, set)]
    pub tiles: Vec<StBytes>,
}

#[pymethods]
impl Dpci {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        let tiles = data
            .chunks(DPCI_TILE_DIM * DPCI_TILE_DIM / 2)
            .map(StBytes::from)
            .collect(); // / 2 because 4bpp
        Ok(Self { tiles })
    }

    /// Convert all individual tiles of the DPCI into one image.
    /// The image contains all tiles next to each other, the image width is tile_width tiles.
    /// The resulting image has one large palette with all palettes merged together.
    ///
    /// palettes is a list of 16 16 color palettes.
    /// The tiles are exported with the first palette in the list of palettes.
    /// The result image contains a palette that consists of all palettes merged together.
    #[pyo3(signature = (palettes, width_in_tiles = 20, palette_index = 0))]
    pub fn tiles_to_pil(
        &self,
        palettes: Vec<Vec<u8>>,
        width_in_tiles: usize,
        palette_index: u8,
    ) -> IndexedImage {
        let tilemap = (0..self.tiles.len()).map(|i| TilemapEntry(i, false, false, palette_index));
        let width = width_in_tiles * DPCI_TILE_DIM;
        let height =
            (((self.tiles.len()) as f32 / width_in_tiles as f32).ceil()) as usize * DPCI_TILE_DIM;
        TiledImage::tiled_to_native(
            tilemap,
            PixelGenerator::tiled4bpp(&self.tiles[..]),
            palettes.iter().flat_map(|x| x.iter().copied()),
            DPCI_TILE_DIM,
            width,
            height,
            1,
        )
    }

    /// Imports tiles that are in a format as described in the documentation for tiles_to_pil.
    pub fn pil_to_tiles(&mut self, image: In256ColIndexedImage, py: Python) -> PyResult<()> {
        let image = image.extract(py)?;
        let w = image.0 .1;
        let h = image.0 .2;
        let (tiles, _) = TiledImage::native_to_tiled_seq(image, DPCI_TILE_DIM, w, h)?;
        self.tiles = tiles.into_iter().map(|x| x.0.into()).collect();
        Ok(())
    }

    /// Replace the tiles.
    /// If contains_null_tile is false, the null tile is added to the list, at the beginning.
    #[pyo3(signature = (tiles, contains_null_tile = false))]
    pub fn import_tiles(&mut self, mut tiles: Vec<StBytes>, contains_null_tile: bool) {
        if !contains_null_tile {
            tiles.insert(0, vec![0; DPCI_TILE_DIM * DPCI_TILE_DIM / 2].into());
        }
        self.tiles = tiles;
    }
}

#[pyclass(module = "skytemple_rust.st_dpci")]
#[derive(Clone, Default)]
pub struct DpciWriter;

#[pymethods]
impl DpciWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<Dpci>, py: Python) -> PyResult<StBytes> {
        Ok(StBytes::from(
            model
                .borrow(py)
                .tiles
                .iter()
                .flat_map(|x| &x.0)
                .copied()
                .collect::<Vec<_>>(),
        ))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_dpci_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_dpci";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dpci>()?;
    m.add_class::<DpciWriter>()?;

    Ok((name, m))
}

/////////////////////////
/////////////////////////
// DPCIs as inputs (for compatibility of including other DPCI implementations from Python)
#[cfg(feature = "python")]
pub mod input {
    use crate::bytes::StBytes;
    use crate::python::*;
    use crate::st_dpci::Dpci;
    use pyo3::types::PyTuple;

    pub trait DpciProvider: ToPyObject {
        fn get_tiles(&self, py: Python) -> PyResult<Vec<StBytes>>;

        fn do_import_tiles(
            &mut self,
            tiles: Vec<StBytes>,
            contains_null_tile: bool,
            py: Python,
        ) -> PyResult<()>;
    }

    impl DpciProvider for Py<Dpci> {
        fn get_tiles(&self, py: Python) -> PyResult<Vec<StBytes>> {
            Ok(self.borrow(py).tiles.clone())
        }

        fn do_import_tiles(
            &mut self,
            tiles: Vec<StBytes>,
            contains_null_tile: bool,
            py: Python,
        ) -> PyResult<()> {
            self.borrow_mut(py).import_tiles(tiles, contains_null_tile);
            Ok(())
        }
    }

    impl DpciProvider for PyObject {
        fn get_tiles(&self, py: Python) -> PyResult<Vec<StBytes>> {
            self.getattr(py, "tiles")?.extract(py)
        }

        fn do_import_tiles(
            &mut self,
            tiles: Vec<StBytes>,
            contains_null_tile: bool,
            py: Python,
        ) -> PyResult<()> {
            let args = PyTuple::new(py, [tiles.into_py(py), contains_null_tile.into_py(py)]);
            self.call_method1(py, "import_tiles", args).map(|_| ())
        }
    }

    pub struct InputDpci(pub Box<dyn DpciProvider>);

    impl<'source> FromPyObject<'source> for InputDpci {
        fn extract(ob: &'source PyAny) -> PyResult<Self> {
            if let Ok(obj) = ob.extract::<Py<Dpci>>() {
                Ok(Self(Box::new(obj)))
            } else {
                Ok(Self(Box::new(ob.to_object(ob.py()))))
            }
        }
    }

    impl IntoPy<PyObject> for InputDpci {
        fn into_py(self, py: Python) -> PyObject {
            self.0.to_object(py)
        }
    }

    impl From<InputDpci> for Dpci {
        fn from(obj: InputDpci) -> Self {
            Python::with_gil(|py| obj.0.to_object(py).extract(py).unwrap())
        }
    }
}

#[cfg(not(feature = "python"))]
pub mod input {
    use crate::bytes::StBytes;
    use crate::no_python::Python;
    use crate::st_dpci::Dpci;
    use crate::PyResult;

    pub trait DpciProvider {
        fn get_tiles(&self, py: Python) -> PyResult<Vec<StBytes>>;

        fn do_import_tiles(
            &mut self,
            tiles: Vec<StBytes>,
            contains_null_tile: bool,
            py: Python,
        ) -> PyResult<()>;
    }

    impl DpciProvider for Dpci {
        fn get_tiles(&self, _py: Python) -> PyResult<Vec<StBytes>> {
            Ok(self.tiles.clone())
        }

        fn do_import_tiles(
            &mut self,
            tiles: Vec<StBytes>,
            contains_null_tile: bool,
            _py: Python,
        ) -> PyResult<()> {
            self.import_tiles(tiles, contains_null_tile);
            Ok(())
        }
    }

    pub struct InputDpci(pub(crate) Dpci);

    impl From<InputDpci> for Dpci {
        fn from(obj: InputDpci) -> Self {
            obj.0
        }
    }
}
