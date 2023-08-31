/*
 * Copyright 2021-2023 Capypara and the SkyTemple Contributors
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
use bytes::{Buf, BufMut};

/// Length of a palette in colors.
pub const DPL_PAL_LEN: usize = 16;
/// Maximum number of palettes
pub const DPL_MAX_PAL: usize = 12;
/// Number of color bytes per palette entry. Fourth is always 0x00.
pub const DPL_PAL_ENTRY_LEN: usize = 4;
/// Size of a single palette in bytes
pub const DPL_PAL_SIZE: usize = DPL_PAL_LEN * DPL_PAL_ENTRY_LEN;
/// The value of the fourth color
pub const DPL_FOURTH_COLOR: u8 = 128;

#[pyclass(module = "skytemple_rust.st_dpl")]
#[derive(Clone)]
pub struct Dpl {
    #[pyo3(get, set)]
    pub palettes: Vec<Vec<u8>>,
}

#[pymethods]
impl Dpl {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        let mut data = &data[..];
        let mut palettes: Vec<Vec<u8>> = Vec::with_capacity(data.len() / DPL_PAL_SIZE);
        let mut current_pal = Vec::with_capacity(16 * 3);
        while data.has_remaining() {
            current_pal.push(data.get_u8());
            current_pal.push(data.get_u8());
            current_pal.push(data.get_u8());
            let unk = data.get_u8();
            debug_assert_eq!(DPL_FOURTH_COLOR, unk);
            if current_pal.len() == DPL_PAL_LEN * 3 {
                palettes.push(current_pal);
                current_pal = Vec::with_capacity(16 * 3);
            }
        }
        if !current_pal.is_empty() {
            palettes.push(current_pal);
        }
        Ok(Self { palettes })
    }
}

#[pyclass(module = "skytemple_rust.st_dpl")]
#[derive(Clone, Default)]
pub struct DplWriter;

#[pymethods]
impl DplWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn write(&self, model: Py<Dpl>, py: Python) -> PyResult<StBytes> {
        let model = model.borrow(py);

        let mut data = Vec::with_capacity(model.palettes.len() * DPL_PAL_SIZE);

        for palette in &model.palettes {
            for (i, color) in palette.iter().enumerate() {
                data.put_u8(*color);
                if i % 3 == 2 {
                    // Insert the fourth color
                    data.put_u8(DPL_FOURTH_COLOR);
                }
            }
        }
        Ok(StBytes::from(data))
    }
}

#[cfg(feature = "python")]
pub(crate) fn create_st_dpl_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_dpl";
    let m = PyModule::new(py, name)?;
    m.add_class::<Dpl>()?;
    m.add_class::<DplWriter>()?;

    Ok((name, m))
}

/////////////////////////
/////////////////////////
// DPLs as inputs (for compatibility of including other DPL implementations from Python)
#[cfg(feature = "python")]
pub mod input {
    use crate::python::*;
    use crate::st_dpl::Dpl;

    pub trait DplProvider: ToPyObject {
        fn set_palettes(&mut self, value: Vec<Vec<u8>>, py: Python) -> PyResult<()>;
    }

    impl DplProvider for Py<Dpl> {
        fn set_palettes(&mut self, value: Vec<Vec<u8>>, py: Python) -> PyResult<()> {
            self.borrow_mut(py).palettes = value;
            Ok(())
        }
    }

    impl DplProvider for PyObject {
        fn set_palettes(&mut self, value: Vec<Vec<u8>>, py: Python) -> PyResult<()> {
            let self_ref = self.as_ref(py);
            self_ref.setattr("palettes", value)
        }
    }

    pub struct InputDpl(pub Box<dyn DplProvider>);

    impl<'source> FromPyObject<'source> for InputDpl {
        fn extract(ob: &'source PyAny) -> PyResult<Self> {
            if let Ok(obj) = ob.extract::<Py<Dpl>>() {
                Ok(Self(Box::new(obj)))
            } else {
                Ok(Self(Box::new(ob.to_object(ob.py()))))
            }
        }
    }

    impl IntoPy<PyObject> for InputDpl {
        fn into_py(self, py: Python) -> PyObject {
            self.0.to_object(py)
        }
    }

    impl From<InputDpl> for Dpl {
        fn from(obj: InputDpl) -> Self {
            Python::with_gil(|py| obj.0.to_object(py).extract(py).unwrap())
        }
    }
}

#[cfg(not(feature = "python"))]
pub mod input {
    use crate::no_python::Python;
    use crate::st_dpl::Dpl;
    use crate::PyResult;

    pub trait DplProvider {
        fn set_palettes(&mut self, value: Vec<Vec<u8>>, py: Python) -> PyResult<()>;
    }

    impl DplProvider for Dpl {
        fn set_palettes(&mut self, value: Vec<Vec<u8>>, _py: Python) -> PyResult<()> {
            self.palettes = value;
            Ok(())
        }
    }

    pub struct InputDpl(pub(crate) Dpl);

    impl From<InputDpl> for Dpl {
        fn from(obj: InputDpl) -> Self {
            obj.0
        }
    }
}
