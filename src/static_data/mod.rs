#[cfg(feature = "python")]
use crate::python::{FromPyObject, PyAny, PyResult};

#[cfg(feature = "python")]
mod python;

/// Input wrapper trait for static ROM config data.
pub struct InStaticData {} // TODO

#[cfg(feature = "python")]
impl<'source> FromPyObject<'source> for InStaticData {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        todo!()
    }
}
