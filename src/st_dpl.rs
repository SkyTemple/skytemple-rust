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

#[pyclass(module = "skytemple_rust.st_dpl")]
#[derive(Clone)]
pub struct Dpl {
    #[pyo3(get, set)]
    pub palettes: Vec<Vec<u8>>,
}

#[pymethods]
impl Dpl {
    #[allow(unused_variables)]
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        todo!()
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
    #[allow(unused_variables)]
    pub fn write(&self, model: Py<Dpl>, py: Python) -> PyResult<StBytes> {
        todo!()
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

    pub trait DplProvider: ToPyObject {}

    impl DplProvider for Py<Dpl> {}

    impl DplProvider for PyObject {}

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
    use crate::st_dpl::Dpl;

    pub trait DplProvider {}

    impl DplProvider for Dpl {}

    pub struct InputDpl(pub(crate) Dpl);

    impl From<InputDpl> for Dpl {
        fn from(obj: InputDpl) -> Self {
            obj.0
        }
    }
}
