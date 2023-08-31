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

//! Module for directly working with ROM files.
//! This is largely based on existing research and a modified version of the nds crate
//! (which only supports extracting to whole directories; no API).

use crate::bytes::StBytes;
use crate::gettext::gettext;
use crate::python::{exceptions, PyResult};
use crate::rom_source::RomFileProvider;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use encoding::all::ISO_8859_1;
use encoding::{EncoderTrap, Encoding};
use itertools::{Either, Itertools};
use memmap2::Mmap;
use nitro_fs::fnt::{Directory, FileEntry};
use nitro_fs::FileSystem;
use num_traits::ToPrimitive;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::iter::once;
use std::path::{Path, PathBuf};

const NOT_ENOUGH_DATA: &str = "The ROM file does not contain enough data.";

enum Header {
    Arm9Offset = 0x20,
    Arm9Len = 0x2C,
    Arm7Offset = 0x30,
    Arm7Len = 0x3C,
    FntOffset = 0x40,
    FntLen = 0x44,
    FatOffset = 0x48,
    FatLen = 0x4C,
    Size = 0x84,
}

/// Provides access to a ROM file system, overlays and ARM9/ARM7 binaries.
pub struct GenericRomFs<T: AsRef<[u8]>> {
    data: T,
    fs: FileSystem,
}

pub type RomFs = GenericRomFs<Mmap>;
pub type WritableRomFs = GenericRomFs<BytesMut>;

impl RomFs {
    pub fn new<P: AsRef<Path>>(path: P, check_crc: bool) -> PyResult<Self> {
        let root = path.as_ref();
        let file = File::open(root)?;
        let data = unsafe { Mmap::map(&file)? };
        Self::create(data, check_crc)
    }
}

impl WritableRomFs {
    pub fn new<P: AsRef<Path>>(path: P, check_crc: bool) -> PyResult<Self> {
        let root = path.as_ref();
        let data = BytesMut::from(&fs::read(root)?[..]);
        Self::create(data, check_crc)
    }
}

impl<T: AsRef<[u8]>> GenericRomFs<T> {
    pub fn create<U>(data: U, check_crc: bool) -> PyResult<Self>
    where
        U: Into<T>,
    {
        let data = data.into();
        pyr_assert!(
            data.as_ref().len() >= 0x160,
            gettext(NOT_ENOUGH_DATA),
            exceptions::PyValueError
        );
        if check_crc {
            let checksum = (&data.as_ref()[0x15E..]).get_u16_le();
            let crc = crc16(&data.as_ref()[0..0x15E]);

            pyr_assert!(
                crc == checksum,
                gettext("ROM Header checksum does not match contents."),
                exceptions::PyValueError
            );
        }

        Ok(Self {
            fs: FileSystem::new(Self::fnt(data.as_ref())?, Self::fat(data.as_ref())?).map_err(
                |e| {
                    exceptions::PyValueError::new_err(gettext!(
                        "Error reading the filesystem from ROM: {}",
                        e.to_string()
                    ))
                },
            )?,
            data,
        })
    }

    pub fn header_bin(&self) -> &[u8] {
        &self.data.as_ref()[0..self.get_u32_le(Header::Size as usize)]
    }

    pub fn arm9_bin(&self) -> &[u8] {
        &self.data.as_ref()[self.get_u32_le(Header::Arm9Offset as usize)
            ..self.get_u32_le(Header::Arm9Len as usize)]
    }

    pub fn arm7_bin(&self) -> &[u8] {
        &self.data.as_ref()[self.get_u32_le(Header::Arm7Offset as usize)
            ..self.get_u32_le(Header::Arm7Len as usize)]
    }

    pub fn get_fs(&self) -> &FileSystem {
        &self.fs
    }

    pub fn get_overlay(&mut self, idx: usize) -> &[u8] {
        let ov = &self.get_fs().overlays()[idx];
        &self.data.as_ref()[ov.alloc.start as usize..ov.alloc.end as usize]
    }

    pub fn get_file_contents(&self, file_id: usize) -> PyResult<&[u8]> {
        let f_entry: &FileEntry = self
            .fs
            .dirs
            .iter()
            .flat_map(|(_, dir)| &dir.files)
            .find(|f| f.id == file_id as u16)
            .ok_or_else(|| exceptions::PyValueError::new_err("The file does not exist."))?;
        assert_eq!(f_entry.id, file_id as u16);
        pyr_assert!(
            f_entry.alloc.end > f_entry.alloc.start,
            "The file is invalid.",
            exceptions::PyValueError
        );
        pyr_assert!(
            (f_entry.alloc.end as usize) < self.data.as_ref().len(),
            "The file is invalid.",
            exceptions::PyValueError
        );
        Ok(&self.data.as_ref()[(f_entry.alloc.start) as usize..(f_entry.alloc.end) as usize])
    }

    /// Writes ROMs. WARNING: This is currently pretty limited. It will fail if the fat/fnt size
    /// changed. This also does not update / work correctly with any potential RSA signature or the like.
    pub fn to_bytes(&self) -> PyResult<StBytes> {
        // Clone data
        let mut data = BytesMut::from(self.data.as_ref());
        let data_len = data.len();
        // ROM size
        (&mut data[0x80..]).put_u32_le(data_len as u32);
        // Recalc crc16
        let crc = crc16(&data.as_ref()[0..0x15E]);
        (&mut data[0x15E..]).put_u16_le(crc);
        // Write fat + fnt
        let (fat, fnt) = self.fat_and_fnt_to_bytes()?;
        let fat_start = Self::get_u32_le_static(&data[..], Header::FatOffset as usize);
        let fat_len = Self::get_u32_le_static(&data[..], Header::FatLen as usize);
        match fat.len().cmp(&fat_len) {
            Ordering::Less => {
                let offset = Header::FatLen as usize;
                (&mut data[offset..offset + 4]).put_u32_le(fat.len() as u32);
                data[fat_start..fat_start + fat.len()].copy_from_slice(&fat[..]);
            }
            Ordering::Equal => data[fat_start..fat_start + fat_len].copy_from_slice(&fat[..]),
            Ordering::Greater => {
                return Err(exceptions::PyValueError::new_err(
                    "The FAT size increased. This is currently not supported.",
                ))
            }
        }
        let fnt_start = Self::get_u32_le_static(&data[..], Header::FntOffset as usize);
        let fnt_len = Self::get_u32_le_static(&data[..], Header::FntLen as usize);
        match fnt.len().cmp(&fnt_len) {
            Ordering::Less => {
                let offset = Header::FntLen as usize;
                (&mut data[offset..offset + 4]).put_u32_le(fnt.len() as u32);
                data[fnt_start..fnt_start + fnt.len()].copy_from_slice(&fnt[..]);
            }
            Ordering::Equal => data[fnt_start..fnt_start + fnt_len].copy_from_slice(&fnt[..]),
            Ordering::Greater => {
                return Err(exceptions::PyValueError::new_err(
                    "The FNT size increased. This is currently not supported.",
                ))
            }
        }
        pyr_assert!(
            fnt.len() == fnt_len,
            "The FNT size increased. This is currently not supported.",
            exceptions::PyValueError
        );
        Ok(StBytes::from(data))
    }

    #[inline(always)]
    fn get_u32_le(&self, offset: usize) -> usize {
        Self::get_u32_le_static(self.data.as_ref(), offset)
    }

    #[inline(always)]
    /// Reads a u32 from `data` at the given offset.
    fn get_u32_le_static(data: &[u8], offset: usize) -> usize {
        (&data[offset..]).get_u32_le() as usize
    }

    pub(crate) fn fat(data: &[u8]) -> PyResult<&[u8]> {
        let fat_start = Self::get_u32_le_static(data, Header::FatOffset as usize);
        let fat_len = Self::get_u32_le_static(data, Header::FatLen as usize);

        pyr_assert!(
            data.len() >= fat_start + fat_len,
            gettext(NOT_ENOUGH_DATA),
            exceptions::PyValueError
        );

        Ok(&data[fat_start..fat_start + fat_len])
    }

    pub(crate) fn fnt(data: &[u8]) -> PyResult<&[u8]> {
        let fnt_start = Self::get_u32_le_static(data, Header::FntOffset as usize);
        let fnt_len = Self::get_u32_le_static(data, Header::FntLen as usize);

        pyr_assert!(
            data.len() >= fnt_start + fnt_len,
            gettext(NOT_ENOUGH_DATA),
            exceptions::PyValueError
        );

        Ok(&data[fnt_start..fnt_start + fnt_len])
    }

    fn fat_and_fnt_to_bytes(&self) -> PyResult<(Bytes, Bytes)> {
        // Part of this was inspired / taken from ndspy.

        let fat = self
            .fs
            .overlays()
            .iter()
            .chain(self.fs.files().iter().cloned())
            .flat_map(|fe| [fe.alloc.start.to_le_bytes(), fe.alloc.end.to_le_bytes()])
            .flatten()
            .collect::<Bytes>();

        fn sub_directories<'a>(
            dirs: &'a BTreeMap<u16, Directory>,
            d: &Directory,
        ) -> Vec<&'a Directory> {
            return dirs
                .values()
                .filter(|id| {
                    id.path
                        .parent()
                        .map_or(false, |idp| idp == d.path.as_path())
                })
                .collect();
        }
        let mut end_cursor: u32 = (self.fs.dirs.len() * 8) as u32;

        let (fnt_header, fnt_content): (Vec<Vec<u8>>, Vec<Vec<u8>>) = self
            .fs
            .dirs
            .values()
            .flat_map(|d| {
                // Create an entries table and add filenames and folders to it
                let entries_table = d
                    .files
                    .iter()
                    .flat_map(|f| {
                        let str = ISO_8859_1
                            .encode(
                                f.path
                                    .file_name()
                                    .unwrap()
                                    .to_os_string()
                                    .into_string()
                                    .unwrap()
                                    .as_str(),
                                EncoderTrap::Replace,
                            )
                            .unwrap();
                        assert!(str.len() < 127);
                        once(str.len().to_u8().unwrap()).chain(str)
                    })
                    .chain(sub_directories(&self.fs.dirs, d).into_iter().flat_map(|f| {
                        let str = ISO_8859_1
                            .encode(
                                f.path
                                    .file_name()
                                    .unwrap()
                                    .to_os_string()
                                    .into_string()
                                    .unwrap()
                                    .as_str(),
                                EncoderTrap::Replace,
                            )
                            .unwrap();
                        assert!(str.len() < 127);
                        once(str.len().to_u8().unwrap() | 0x80)
                            .chain(str)
                            .chain(f.id().to_le_bytes())
                    }))
                    .chain(
                        // And the entries table needs to end with a null byte to mark
                        // its end.
                        once(0),
                    )
                    .collect::<Vec<u8>>();

                let header: Vec<u8> = end_cursor
                    .to_le_bytes()
                    .iter()
                    .chain(d.start_id().to_le_bytes().iter())
                    .chain(
                        if d.is_root() {
                            self.fs.dirs.len() as u16
                        } else {
                            d.parent_id()
                        }
                        .to_le_bytes()
                        .iter(),
                    )
                    .copied()
                    .collect();
                end_cursor += entries_table.len() as u32;
                [header, entries_table]
            })
            .enumerate()
            .partition_map(|(id, v)| {
                if id % 2 == 0 {
                    Either::Left(v)
                } else {
                    Either::Right(v)
                }
            });
        Ok((
            fat,
            fnt_header
                .into_iter()
                .flatten()
                .chain(fnt_content.into_iter().flatten())
                .collect(),
        ))
    }
}

impl<T: AsRef<[u8]>> RomFileProvider for GenericRomFs<T> {
    fn get_file_by_name(&self, filename: &str) -> PyResult<Vec<u8>> {
        let filename = PathBuf::from(filename);
        let files =
            self._list_files_in_folder(filename.parent().unwrap_or_else(|| Path::new("")))?;
        if let Some(file) = files.iter().find(|e| e.path == filename) {
            Ok(self.get_file_contents(file.id as usize)?.to_vec())
        } else {
            Err(exceptions::PyValueError::new_err("File not found."))
        }
    }
    fn list_files_in_folder(&self, filename: &str) -> PyResult<Vec<String>> {
        Ok(self
            ._list_files_in_folder(filename)?
            .iter()
            .map(|f| {
                f.path
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap()
            })
            .collect::<Vec<String>>())
    }
}

impl<T: AsRef<[u8]>> GenericRomFs<T> {
    fn _list_files_in_folder<U: Into<PathBuf>>(&self, path: U) -> PyResult<&Vec<FileEntry>> {
        let path = path.into();
        for dir in self.fs.dirs.values() {
            if dir.path == path {
                return Ok(&dir.files);
            }
        }
        Err(exceptions::PyValueError::new_err("Directory not found."))
    }
}

impl WritableRomFs {
    pub fn header_bin_mut(&mut self) -> &mut [u8] {
        let end = self.get_u32_le(Header::Size as usize);
        &mut self.data.as_mut()[0..end]
    }

    pub fn arm9_bin_mut(&mut self) -> &mut [u8] {
        let begin = self.get_u32_le(Header::Arm9Offset as usize);
        let end = self.get_u32_le(Header::Arm9Len as usize);
        &mut self.data.as_mut()[begin..end]
    }

    pub fn arm7_bin_mut(&mut self) -> &mut [u8] {
        let begin = self.get_u32_le(Header::Arm7Offset as usize);
        let end = self.get_u32_le(Header::Arm7Len as usize);
        &mut self.data.as_mut()[begin..end]
    }

    pub fn get_fs_mut(&mut self) -> &mut FileSystem {
        &mut self.fs
    }

    pub fn get_overlay_mut(&mut self, idx: usize) -> &mut [u8] {
        let ov = &self.get_fs().overlays()[idx];
        let start = ov.alloc.start as usize;
        let end = ov.alloc.end as usize;
        &mut self.data.as_mut()[start..end]
    }

    pub fn set_file_by_name(&mut self, filename: &str, content: StBytes) -> PyResult<()> {
        let filename = PathBuf::from(filename);
        let files =
            self._list_files_in_folder(filename.parent().unwrap_or_else(|| Path::new("")))?;
        if let Some(file) = files.iter().find(|e| e.path == filename) {
            let file_id = file.id as usize;
            self.set_file_contents(file_id, content)
        } else {
            Err(exceptions::PyValueError::new_err("File not found."))
        }
    }

    pub fn set_file_contents(&mut self, file_id: usize, content: StBytes) -> PyResult<()> {
        let f_entry: &mut FileEntry = self
            .fs
            .dirs
            .iter_mut()
            .flat_map(|(_, dir)| &mut dir.files)
            .find(|f| f.id == file_id as u16)
            .ok_or_else(|| exceptions::PyValueError::new_err("The file does not exist."))?;
        pyr_assert!(
            f_entry.alloc.end > f_entry.alloc.start,
            "The file is invalid.",
            exceptions::PyValueError
        );
        pyr_assert!(
            (f_entry.alloc.end as usize) < self.data.as_ref().len(),
            "The file is invalid.",
            exceptions::PyValueError
        );

        let old_len = f_entry.alloc.end - f_entry.alloc.start;
        let old_end = f_entry.alloc.end;
        f_entry.alloc.end = f_entry.alloc.start + (content.len() as u32);
        if content.len() as u32 > old_len {
            let increased_size = content.len() as u32 - old_len;
            // Add content
            self.data = self
                .data
                .iter()
                .copied()
                .take(f_entry.alloc.start as usize)
                .chain(content)
                .chain(self.data.iter().skip(old_end as usize).copied())
                .collect();
            // Starts and ends
            self.fs
                .dirs
                .iter_mut()
                .flat_map(|(_, dir)| &mut dir.files)
                .filter(|f| f.alloc.start >= old_end)
                .for_each(|f| {
                    f.alloc.start += increased_size;
                    f.alloc.end += increased_size;
                });
        } else {
            self.data[(f_entry.alloc.start as usize)..(f_entry.alloc.end as usize)]
                .copy_from_slice(&content[..]);
        }
        Ok(())
    }
}

impl Clone for WritableRomFs {
    fn clone(&self) -> Self {
        Self {
            fs: self.fs.clone(),
            data: self.data.clone(),
        }
    }
}

static CRC16_TABLE: [u16; 256] = [
    0x0000, 0xC0C1, 0xC181, 0x0140, 0xC301, 0x03C0, 0x0280, 0xC241, 0xC601, 0x06C0, 0x0780, 0xC741,
    0x0500, 0xC5C1, 0xC481, 0x0440, 0xCC01, 0x0CC0, 0x0D80, 0xCD41, 0x0F00, 0xCFC1, 0xCE81, 0x0E40,
    0x0A00, 0xCAC1, 0xCB81, 0x0B40, 0xC901, 0x09C0, 0x0880, 0xC841, 0xD801, 0x18C0, 0x1980, 0xD941,
    0x1B00, 0xDBC1, 0xDA81, 0x1A40, 0x1E00, 0xDEC1, 0xDF81, 0x1F40, 0xDD01, 0x1DC0, 0x1C80, 0xDC41,
    0x1400, 0xD4C1, 0xD581, 0x1540, 0xD701, 0x17C0, 0x1680, 0xD641, 0xD201, 0x12C0, 0x1380, 0xD341,
    0x1100, 0xD1C1, 0xD081, 0x1040, 0xF001, 0x30C0, 0x3180, 0xF141, 0x3300, 0xF3C1, 0xF281, 0x3240,
    0x3600, 0xF6C1, 0xF781, 0x3740, 0xF501, 0x35C0, 0x3480, 0xF441, 0x3C00, 0xFCC1, 0xFD81, 0x3D40,
    0xFF01, 0x3FC0, 0x3E80, 0xFE41, 0xFA01, 0x3AC0, 0x3B80, 0xFB41, 0x3900, 0xF9C1, 0xF881, 0x3840,
    0x2800, 0xE8C1, 0xE981, 0x2940, 0xEB01, 0x2BC0, 0x2A80, 0xEA41, 0xEE01, 0x2EC0, 0x2F80, 0xEF41,
    0x2D00, 0xEDC1, 0xEC81, 0x2C40, 0xE401, 0x24C0, 0x2580, 0xE541, 0x2700, 0xE7C1, 0xE681, 0x2640,
    0x2200, 0xE2C1, 0xE381, 0x2340, 0xE101, 0x21C0, 0x2080, 0xE041, 0xA001, 0x60C0, 0x6180, 0xA141,
    0x6300, 0xA3C1, 0xA281, 0x6240, 0x6600, 0xA6C1, 0xA781, 0x6740, 0xA501, 0x65C0, 0x6480, 0xA441,
    0x6C00, 0xACC1, 0xAD81, 0x6D40, 0xAF01, 0x6FC0, 0x6E80, 0xAE41, 0xAA01, 0x6AC0, 0x6B80, 0xAB41,
    0x6900, 0xA9C1, 0xA881, 0x6840, 0x7800, 0xB8C1, 0xB981, 0x7940, 0xBB01, 0x7BC0, 0x7A80, 0xBA41,
    0xBE01, 0x7EC0, 0x7F80, 0xBF41, 0x7D00, 0xBDC1, 0xBC81, 0x7C40, 0xB401, 0x74C0, 0x7580, 0xB541,
    0x7700, 0xB7C1, 0xB681, 0x7640, 0x7200, 0xB2C1, 0xB381, 0x7340, 0xB101, 0x71C0, 0x7080, 0xB041,
    0x5000, 0x90C1, 0x9181, 0x5140, 0x9301, 0x53C0, 0x5280, 0x9241, 0x9601, 0x56C0, 0x5780, 0x9741,
    0x5500, 0x95C1, 0x9481, 0x5440, 0x9C01, 0x5CC0, 0x5D80, 0x9D41, 0x5F00, 0x9FC1, 0x9E81, 0x5E40,
    0x5A00, 0x9AC1, 0x9B81, 0x5B40, 0x9901, 0x59C0, 0x5880, 0x9841, 0x8801, 0x48C0, 0x4980, 0x8941,
    0x4B00, 0x8BC1, 0x8A81, 0x4A40, 0x4E00, 0x8EC1, 0x8F81, 0x4F40, 0x8D01, 0x4DC0, 0x4C80, 0x8C41,
    0x4400, 0x84C1, 0x8581, 0x4540, 0x8701, 0x47C0, 0x4680, 0x8641, 0x8201, 0x42C0, 0x4380, 0x8341,
    0x4100, 0x81C1, 0x8081, 0x4040,
];

pub fn crc16(data: &[u8]) -> u16 {
    data.iter().fold(0xFFFF, |crc, byte| {
        (crc >> 8) ^ CRC16_TABLE[(crc as u8 ^ *byte) as usize]
    })
}

// #[cfg(test)]
// #[test]
// fn test_rom_fat_fnt_save() {
//     let in_rom = WritableRomFs::new("/tmp/rom.nds", true).unwrap();
//
//     let in_fat = WritableRomFs::fat(&in_rom.data).unwrap();
//     let in_fnt = WritableRomFs::fnt(&in_rom.data).unwrap();
//
//     let (out_fat, out_fnt) = in_rom.fat_and_fnt_to_bytes().unwrap();
//
//     assert_eq!(in_fat.len(), out_fat.len());
//     assert_eq!(in_fat, &out_fat[..]);
//     assert_eq!(in_fnt.len(), out_fnt.len());
//
//     std::fs::write("/tmp/a.fnt", &in_fnt).unwrap();
//     std::fs::write("/tmp/b.fnt", &out_fnt[..]).unwrap();
//
//     assert_eq!(in_fnt, &out_fnt[..]);
//
//     let out_rom = WritableRomFs::create(BytesMut::from(&in_rom.to_bytes().unwrap()[..]), true).unwrap();
//     assert_eq!(in_rom.fs, out_rom.fs);
// }
