use crate::bytes::StBytes;
use crate::python::PyErr;
use crate::static_data::InStaticData;
use thiserror::Error;

pub type Sir0Result<T> = Result<T, Sir0Error>;

#[derive(Error, Debug, Clone)]
pub enum Sir0Error {
    #[error("dummy todo")]
    Dummy,
}

impl From<Sir0Error> for PyErr {
    fn from(source: Sir0Error) -> Self {
        todo!()
    }
}

pub trait Sir0Serializable
where
    Self: Sized,
{
    fn sir0_serialize_parts(&self) -> Sir0Result<(StBytes, Vec<usize>, Option<usize>)>;

    fn sir0_unwrap(
        content_data: StBytes,
        data_pointer: usize,
        static_data: InStaticData,
    ) -> Sir0Result<Self>;
}
