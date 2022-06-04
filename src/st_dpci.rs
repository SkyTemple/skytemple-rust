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

#[pyclass(module = "skytemple_rust.st_dpci")]
#[derive(Clone)]
pub struct Dpci {}

#[pymethods]
impl Dpci {
    #[allow(unused_variables)]
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        todo!()
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
    #[allow(unused_variables)]
    pub fn write(&self, model: Py<Dpci>, py: Python) -> PyResult<StBytes> {
        todo!()
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
    use crate::python::*;
    use crate::st_dpci::Dpci;

    pub trait DpciProvider: ToPyObject {}

    impl DpciProvider for Py<Dpci> {}

    impl DpciProvider for PyObject {}

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
    use crate::st_dpci::Dpci;

    pub trait DpciProvider {}

    impl DpciProvider for Dpci {}

    pub struct InputDpci(pub(crate) Dpci);

    impl From<InputDpci> for Dpci {
        fn from(obj: InputDpci) -> Self {
            obj.0
        }
    }
}
