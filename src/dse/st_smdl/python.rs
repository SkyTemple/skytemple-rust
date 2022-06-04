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
use crate::dse::date::DseDate;
use crate::dse::filename::DseFilename;
use crate::python::*;
use num_traits::FromPrimitive;
use pyo3::types::PyList;

mod implem {
    pub use crate::dse::st_smdl::eoc;
    pub use crate::dse::st_smdl::event;
    pub use crate::dse::st_smdl::smdl;
    pub use crate::dse::st_smdl::song;
    pub use crate::dse::st_smdl::trk;
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlHeader {
    #[pyo3(get, set)]
    version: u16,
    #[pyo3(get, set)]
    unk1: u8,
    #[pyo3(get, set)]
    unk2: u8,
    #[pyo3(get, set)]
    modified_date: StBytes,
    #[pyo3(get, set)]
    file_name: StBytes,
    #[pyo3(get, set)]
    unk5: u32,
    #[pyo3(get, set)]
    unk6: u32,
    #[pyo3(get, set)]
    unk8: u32,
    #[pyo3(get, set)]
    unk9: u32,
}

impl From<implem::smdl::SmdlHeader> for SmdlHeader {
    fn from(source: implem::smdl::SmdlHeader) -> Self {
        SmdlHeader {
            version: source.version,
            unk1: source.unk1,
            unk2: source.unk2,
            modified_date: source.modified_date.into(),
            file_name: source.file_name.into(),
            unk5: source.unk5,
            unk6: source.unk6,
            unk8: source.unk8,
            unk9: source.unk9,
        }
    }
}

impl From<SmdlHeader> for implem::smdl::SmdlHeader {
    fn from(mut source: SmdlHeader) -> Self {
        implem::smdl::SmdlHeader {
            version: source.version,
            unk1: source.unk1,
            unk2: source.unk2,
            modified_date: DseDate::from(&mut source.modified_date),
            file_name: DseFilename::from(&mut source.file_name.0),
            unk5: source.unk5,
            unk6: source.unk6,
            unk8: source.unk8,
            unk9: source.unk9,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlSong {
    #[pyo3(get, set)]
    unk1: u32,
    #[pyo3(get, set)]
    unk2: u32,
    #[pyo3(get, set)]
    unk3: u32,
    #[pyo3(get, set)]
    unk4: u16,
    #[pyo3(get, set)]
    tpqn: u16,
    #[pyo3(get, set)]
    unk5: u16,
    #[pyo3(get, set)]
    nbchans: u8,
    #[pyo3(get, set)]
    unk6: u32,
    #[pyo3(get, set)]
    unk7: u32,
    #[pyo3(get, set)]
    unk8: u32,
    #[pyo3(get, set)]
    unk9: u32,
    #[pyo3(get, set)]
    unk10: u16,
    #[pyo3(get, set)]
    unk11: u16,
    #[pyo3(get, set)]
    unk12: u32,
}

impl From<implem::song::SmdlSong> for SmdlSong {
    fn from(source: implem::song::SmdlSong) -> Self {
        SmdlSong {
            unk1: source.unk1,
            unk2: source.unk2,
            unk3: source.unk3,
            unk4: source.unk4,
            tpqn: source.tpqn,
            unk5: source.unk5,
            nbchans: source.nbchans,
            unk6: source.unk6,
            unk7: source.unk7,
            unk8: source.unk8,
            unk9: source.unk9,
            unk10: source.unk10,
            unk11: source.unk11,
            unk12: source.unk12,
        }
    }
}

impl From<SmdlSong> for implem::song::SmdlSong {
    fn from(source: SmdlSong) -> Self {
        implem::song::SmdlSong::new(
            source.unk1,
            source.unk2,
            source.unk3,
            source.unk4,
            source.tpqn,
            source.unk5,
            source.nbchans,
            source.unk6,
            source.unk7,
            source.unk8,
            source.unk9,
            source.unk10,
            source.unk11,
            source.unk12,
        )
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlEoc {
    #[pyo3(get, set)]
    param1: u32,
    #[pyo3(get, set)]
    param2: u32,
}

impl From<implem::eoc::SmdlEoc> for SmdlEoc {
    fn from(source: implem::eoc::SmdlEoc) -> Self {
        SmdlEoc {
            param1: source.param1,
            param2: source.param2,
        }
    }
}

impl From<SmdlEoc> for implem::eoc::SmdlEoc {
    fn from(source: SmdlEoc) -> Self {
        implem::eoc::SmdlEoc {
            param1: source.param1,
            param2: source.param2,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlTrackHeader {
    #[pyo3(get, set)]
    param1: u32,
    #[pyo3(get, set)]
    param2: u32,
}

impl From<implem::trk::SmdlTrackHeader> for SmdlTrackHeader {
    fn from(source: implem::trk::SmdlTrackHeader) -> Self {
        SmdlTrackHeader {
            param1: source.param1,
            param2: source.param2,
        }
    }
}

impl From<SmdlTrackHeader> for implem::trk::SmdlTrackHeader {
    fn from(source: SmdlTrackHeader) -> Self {
        implem::trk::SmdlTrackHeader::new(source.param1, source.param2)
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlTrackPreamble {
    #[pyo3(get, set)]
    track_id: u8,
    #[pyo3(get, set)]
    channel_id: u8,
    #[pyo3(get, set)]
    unk1: u8,
    #[pyo3(get, set)]
    unk2: u8,
}

impl From<implem::trk::SmdlTrackPreamble> for SmdlTrackPreamble {
    fn from(source: implem::trk::SmdlTrackPreamble) -> Self {
        SmdlTrackPreamble {
            track_id: source.track_id,
            channel_id: source.channel_id,
            unk1: source.unk1,
            unk2: source.unk2,
        }
    }
}

impl From<SmdlTrackPreamble> for implem::trk::SmdlTrackPreamble {
    fn from(source: SmdlTrackPreamble) -> Self {
        implem::trk::SmdlTrackPreamble {
            track_id: source.track_id,
            channel_id: source.channel_id,
            unk1: source.unk1,
            unk2: source.unk2,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlEventPlayNote {
    #[pyo3(get, set)]
    velocity: u8,
    #[pyo3(get, set)]
    octave_mod: i8,
    #[pyo3(get, set)]
    note: u8,
    #[pyo3(get, set)]
    key_down_duration: Option<u32>,
}

impl SmdlEventPlayNote {
    pub fn new(
        note: implem::event::SmdlNote,
        octave_mod: i8,
        velocity: u8,
        key_down_duration: Option<u32>,
    ) -> Self {
        SmdlEventPlayNote {
            velocity,
            octave_mod,
            note: note as u8,
            key_down_duration,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlEventPause {
    #[pyo3(get, set)]
    value: u8,
}

impl SmdlEventPause {
    pub fn new(value: implem::event::SmdlPause) -> Self {
        SmdlEventPause { value: value as u8 }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlEventSpecial {
    #[pyo3(get, set)]
    op: u8,
    #[pyo3(get, set)]
    params: Vec<u8>,
}

impl SmdlEventSpecial {
    pub fn new(op: implem::event::SmdlSpecialOpCode, params: Vec<u8>) -> Self {
        SmdlEventSpecial {
            op: op as u8,
            params,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlTrack {
    #[pyo3(get, set)]
    header: Py<SmdlTrackHeader>,
    #[pyo3(get, set)]
    preamble: Py<SmdlTrackPreamble>,
    #[pyo3(get, set)]
    events: Py<PyList>, // Sequence[Union[SmdlEventSpecial, SmdlEventPause, SmdlEventPlayNote]]
}

impl From<implem::trk::SmdlTrack> for SmdlTrack {
    fn from(source: implem::trk::SmdlTrack) -> Self {
        Python::with_gil(|py| {
            let events = Py::from(PyList::new(
                py,
                source.events.into_iter().map(|e| match e {
                    implem::event::SmdlEvent::Special { op, params } => {
                        SmdlEventSpecial::new(op, params).into_py(py)
                    }
                    implem::event::SmdlEvent::Pause { value } => {
                        SmdlEventPause::new(value).into_py(py)
                    }
                    implem::event::SmdlEvent::Note {
                        note,
                        octave_mod,
                        velocity,
                        key_down_duration,
                    } => SmdlEventPlayNote::new(note, octave_mod, velocity, key_down_duration)
                        .into_py(py),
                }),
            ));
            SmdlTrack {
                header: Py::new(py, SmdlTrackHeader::from(source.header)).unwrap(),
                preamble: Py::new(py, SmdlTrackPreamble::from(source.preamble)).unwrap(),
                events,
            }
        })
    }
}

impl From<SmdlTrack> for implem::trk::SmdlTrack {
    fn from(source: SmdlTrack) -> Self {
        Python::with_gil(|py| {
            let events = source
                .events
                .extract::<&PyList>(py)
                .unwrap()
                .into_iter()
                .map(|e| {
                    if let Ok(v) = e.extract::<SmdlEventSpecial>() {
                        implem::event::SmdlEvent::Special {
                            op: implem::event::SmdlSpecialOpCode::from_u8(v.op)
                                .expect("Invalid special opcode."),
                            params: v.params,
                        }
                    } else if let Ok(v) = e.extract::<SmdlEventPause>() {
                        implem::event::SmdlEvent::Pause {
                            value: implem::event::SmdlPause::from_u8(v.value)
                                .expect("Invalid pause opcode."),
                        }
                    } else if let Ok(v) = e.extract::<SmdlEventPlayNote>() {
                        implem::event::SmdlEvent::Note {
                            note: implem::event::SmdlNote::from_u8(v.note)
                                .expect("Invalid note opcode."),
                            octave_mod: v.octave_mod,
                            velocity: v.velocity,
                            key_down_duration: v.key_down_duration,
                        }
                    } else {
                        panic!("Invalid event: {:?}", e)
                    }
                })
                .collect();
            implem::trk::SmdlTrack {
                header: source.header.extract::<SmdlTrackHeader>(py).unwrap().into(),
                preamble: source
                    .preamble
                    .extract::<SmdlTrackPreamble>(py)
                    .unwrap()
                    .into(),
                events,
            }
        })
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct Smdl {
    #[pyo3(get, set)]
    header: Py<SmdlHeader>,
    #[pyo3(get, set)]
    song: Py<SmdlSong>,
    #[pyo3(get, set)]
    tracks: Vec<Py<SmdlTrack>>,
    #[pyo3(get, set)]
    eoc: Py<SmdlEoc>,
}

#[pymethods]
impl Smdl {
    #[new]
    pub fn new(data: StBytes) -> PyResult<Self> {
        Ok(<PyResult<implem::smdl::Smdl>>::from(data)?.into())
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone, Default)]
pub(crate) struct SmdlWriter;

#[pymethods]
impl SmdlWriter {
    #[new]
    pub fn new() -> Self {
        Self
    }
    pub fn write(&self, model: Smdl) -> StBytes {
        implem::smdl::Smdl::from(model).into()
    }
}

impl From<implem::smdl::Smdl> for Smdl {
    fn from(source: implem::smdl::Smdl) -> Self {
        Python::with_gil(|py| Smdl {
            header: Py::new(py, SmdlHeader::from(source.header)).unwrap(),
            song: Py::new(py, SmdlSong::from(source.song)).unwrap(),
            tracks: source
                .tracks
                .into_iter()
                .map(|track| Py::new(py, SmdlTrack::from(track)).unwrap())
                .collect(),
            eoc: Py::new(py, SmdlEoc::from(source.eoc)).unwrap(),
        })
    }
}

impl From<Smdl> for implem::smdl::Smdl {
    fn from(source: Smdl) -> Self {
        Python::with_gil(|py| implem::smdl::Smdl {
            header: source.header.extract::<SmdlHeader>(py).unwrap().into(),
            song: source.song.extract::<SmdlSong>(py).unwrap().into(),
            tracks: source
                .tracks
                .into_iter()
                .map(|track| track.extract::<SmdlTrack>(py).unwrap().into())
                .collect(),
            eoc: source.eoc.extract::<SmdlEoc>(py).unwrap().into(),
        })
    }
}

pub(crate) fn create_st_smdl_module(py: Python) -> PyResult<(&str, &PyModule)> {
    let name: &'static str = "skytemple_rust.st_smdl";
    let m = PyModule::new(py, name)?;
    m.add_class::<Smdl>()?;
    m.add_class::<SmdlWriter>()?;

    Ok((name, m))
}
