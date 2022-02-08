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

use pyo3::types::PyList;
use crate::bytes::StBytes;
use crate::python::*;
use crate::dse::st_smdl::model as implem;

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlHeader {
    #[pyo3(get, set)]
    version: u32,
    #[pyo3(get, set)]
    unk1: u32,
    #[pyo3(get, set)]
    unk2: u32,
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

impl From<implem::SmdlHeader> for SmdlHeader {
    fn from(source: implem::SmdlHeader) -> Self {
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

impl From<SmdlHeader> for implem::SmdlHeader {
    fn from(source: SmdlHeader) -> Self {
        implem::SmdlHeader {
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
    unk4: u32,
    #[pyo3(get, set)]
    tpqn: u32,
    #[pyo3(get, set)]
    unk5: u32,
    #[pyo3(get, set)]
    nbchans: u32,
    #[pyo3(get, set)]
    unk6: u32,
    #[pyo3(get, set)]
    unk7: u32,
    #[pyo3(get, set)]
    unk8: u32,
    #[pyo3(get, set)]
    unk9: u32,
    #[pyo3(get, set)]
    unk10: u32,
    #[pyo3(get, set)]
    unk11: u32,
    #[pyo3(get, set)]
    unk12: u32,
}

impl From<implem::SmdlSong> for SmdlSong {
    fn from(source: implem::SmdlSong) -> Self {
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

impl From<SmdlSong> for implem::SmdlSong {
    fn from(source: SmdlSong) -> Self {
        implem::SmdlSong {
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

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlEoc {
    #[pyo3(get, set)]
    param1: u32,
    #[pyo3(get, set)]
    param2: u32,
}

impl From<implem::SmdlEoc> for SmdlEoc {
    fn from(source: implem::SmdlEoc) -> Self {
        SmdlEoc {
            param1: source.param1,
            param2: source.param2,
        }
    }
}

impl From<SmdlEoc> for implem::SmdlEoc {
    fn from(source: SmdlEoc) -> Self {
        implem::SmdlEoc {
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

impl From<implem::SmdlTrackHeader> for SmdlTrackHeader {
    fn from(source: implem::SmdlTrackHeader) -> Self {
        SmdlTrackHeader {
            param1: source.param1,
            param2: source.param2,
        }
    }
}

impl From<SmdlTrackHeader> for implem::SmdlTrackHeader {
    fn from(source: SmdlTrackHeader) -> Self {
        implem::SmdlTrackHeader {
            param1: source.param1,
            param2: source.param2,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlTrackPreamble {
    #[pyo3(get, set)]
    track_id: u32,
    #[pyo3(get, set)]
    channel_id: u32,
    #[pyo3(get, set)]
    unk1: u32,
    #[pyo3(get, set)]
    unk2: u32,
}

impl From<implem::SmdlTrackPreamble> for SmdlTrackPreamble {
    fn from(source: implem::SmdlTrackPreamble) -> Self {
        SmdlTrackPreamble {
            track_id: source.track_id,
            channel_id: source.channel_id,
            unk1: source.unk1,
            unk2: source.unk2,
        }
    }
}

impl From<SmdlTrackPreamble> for implem::SmdlTrackPreamble {
    fn from(source: SmdlTrackPreamble) -> Self {
        implem::SmdlTrackPreamble {
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
    velocity: u32,
    #[pyo3(get, set)]
    octave_mod: u32,
    #[pyo3(get, set)]
    note: u32,
    #[pyo3(get, set)]
    key_down_duration: Option<u32>,
}

impl From<implem::SmdlEventPlayNote> for SmdlEventPlayNote {
    fn from(source: implem::SmdlEventPlayNote) -> Self {
        SmdlEventPlayNote {
            velocity: source.velocity,
            octave_mod: source.octave_mod,
            note: source.note,
            key_down_duration: source.key_down_duration,
        }
    }
}

impl From<SmdlEventPlayNote> for implem::SmdlEventPlayNote {
    fn from(source: SmdlEventPlayNote) -> Self {
        implem::SmdlEventPlayNote {
            velocity: source.velocity,
            octave_mod: source.octave_mod,
            note: source.note,
            key_down_duration: source.key_down_duration,
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlEventPause {
    #[pyo3(get, set)]
    value: u32,
}

impl From<implem::SmdlEventPause> for SmdlEventPause {
    fn from(source: implem::SmdlEventPause) -> Self {
        SmdlEventPause {
            value: source.value
        }
    }
}

impl From<SmdlEventPause> for implem::SmdlEventPause {
    fn from(source: SmdlEventPause) -> Self {
        implem::SmdlEventPause {
            value: source.value
        }
    }
}

#[pyclass(module = "skytemple_rust.st_smdl")]
#[derive(Clone)]
pub(crate) struct SmdlEventSpecial {
    #[pyo3(get, set)]
    op: u32,
    #[pyo3(get, set)]
    params: Vec<u32>,
}

impl From<implem::SmdlEventSpecial> for SmdlEventSpecial {
    fn from(source: implem::SmdlEventSpecial) -> Self {
        SmdlEventSpecial {
            op: source.op,
            params: source.params,
        }
    }
}

impl From<SmdlEventSpecial> for implem::SmdlEventSpecial {
    fn from(source: SmdlEventSpecial) -> Self {
        implem::SmdlEventSpecial {
            op: source.op,
            params: source.params,
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
    events: Py<PyList>,  // Sequence[Union[SmdlEventSpecial, SmdlEventPause, SmdlEventPlayNote]]
}

impl From<implem::SmdlTrack> for SmdlTrack {
    fn from(source: implem::SmdlTrack) -> Self {
        Python::with_gil(|py| {
            let events = Py::from(PyList::new(py, source.events.into_iter().map(|e| match e {
                implem::SmdlEvent::Special(e) => SmdlEventSpecial::from(e).into_py(py),
                implem::SmdlEvent::Pause(e) => SmdlEventPause::from(e).into_py(py),
                implem::SmdlEvent::Note(e) => SmdlEventPlayNote::from(e).into_py(py),
            })));
            SmdlTrack {
                header: Py::new(py, SmdlTrackHeader::from(source.header)).unwrap(),
                preamble: Py::new(py, SmdlTrackPreamble::from(source.preamble)).unwrap(),
                events,
            }
        })
    }
}

impl From<SmdlTrack> for implem::SmdlTrack {
    fn from(source: SmdlTrack) -> Self {
        Python::with_gil(|py| {
            let events = source.events.extract::<PyObject>(py).into_iter().map(|e| {
                if let Ok(v) = e.extract::<SmdlEventSpecial>(py) {
                    implem::SmdlEvent::Special(v.into())
                } else if let Ok(v) = e.extract::<SmdlEventPause>(py) {
                    implem::SmdlEvent::Pause(v.into())
                } else if let Ok(v) = e.extract::<SmdlEventPlayNote>(py) {
                    implem::SmdlEvent::Note(v.into())
                } else {
                    panic!("Invalid event.")
                }
            }).collect();
            implem::SmdlTrack {
                header: source.header.extract::<SmdlTrackHeader>(py).unwrap().into(),
                preamble: source.preamble.extract::<SmdlTrackPreamble>(py).unwrap().into(),
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
    eoc: Py<SmdlEoc>
}

#[pymethods]
impl Smdl {
    #[new]
    pub fn new(data: StBytes) -> Self {
        implem::Smdl::from(data).into()
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
    pub fn write(&self, model: Smdl, py: Python) -> StBytes {
        implem::Smdl::from(model).into()
    }
}

impl From<implem::Smdl> for Smdl {
    fn from(source: implem::Smdl) -> Self {
        Python::with_gil(|py| {
            Smdl {
                header: Py::new(py, SmdlHeader::from(source.header)).unwrap(),
                song: Py::new(py, SmdlSong::from(source.song)).unwrap(),
                tracks: source.tracks.into_iter().map(|track| Py::new(py, SmdlTrack::from(track)).unwrap()).collect(),
                eoc: Py::new(py, SmdlEoc::from(source.eoc)).unwrap(),
            }
        })
    }
}

impl From<Smdl> for implem::Smdl {
    fn from(source: Smdl) -> Self {
        Python::with_gil(|py| {
            implem::Smdl {
                header: source.header.extract::<SmdlHeader>(py).unwrap().into(),
                song: source.song.extract::<SmdlSong>(py).unwrap().into(),
                tracks: source.tracks.into_iter().map(|track| track.extract::<SmdlTrack>(py).unwrap().into()).collect(),
                eoc: source.eoc.extract::<SmdlEoc>(py).unwrap().into(),
            }
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