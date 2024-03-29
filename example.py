# mypy: ignore-errors
import os
import sys
from glob import glob
from typing import Optional

from PIL import Image
from skytemple_rust.pmd_wan import Fragment, ImageBytes, FragmentResolution, Frame

from skytemple_rust import pmd_wan
from skytemple_files.common.types.file_types import FileType


PACK_FILE = '/home/marco/austausch/dev/skytemple/ppmd_statsutil/sky_rom/data/MONSTER/monster.bin'
WAN_FILE_PATTERN = '/home/marco/austausch/dev/skytemple/ppmd_statsutil/sky_rom/data/GROUND/*.wan'

for gptrn in glob(WAN_FILE_PATTERN):
    basename = os.path.basename(gptrn)
    print(basename)
    with open(gptrn, 'rb') as f:
        try:
            image = pmd_wan.WanImage(f.read())

            fragment: Fragment
            frame: Frame
            for mfg_i, frame in enumerate(image.meta_frame_store.meta_frame_groups):
                os.makedirs(f'/tmp/outimg/{basename}/{mfg_i}', exist_ok=True)
                for mf_i, meta_frame_id in enumerate(frame.meta_frames_id):
                    fragment = image.meta_frame_store.meta_frames[meta_frame_id]
                    meta_frame_img_bytes: ImageBytes = image.image_store.images[fragment.image_index]
                    resolution: FragmentResolution = fragment.resolution
                    try:
                        im = Image.frombuffer('RGBA',
                                              (resolution.x, resolution.y),
                                              bytearray(meta_frame_img_bytes.to_image(image.palette, fragment)),
                                              'raw', 'RGBA', 0, 1)

                        im.save(f'/tmp/outimg/{basename}/{mfg_i}/{mf_i}.png')
                    except ValueError as e:
                        print(f"Error converting {basename}/{mfg_i}/{mf_i}: {e}", file=sys.stderr)
        except ValueError as e:
            print(f"Error converting {basename}/{mfg_i}/{mf_i}: {e}", file=sys.stderr)

with open(PACK_FILE, 'rb') as f:
    bin_pack = FileType.BIN_PACK.deserialize(f.read())
    for s_i, sprite in enumerate(bin_pack):
        print(f"Poké {s_i}")
        sprite_bin_decompressed = FileType.PKDPX.deserialize(sprite).decompress()
        image = pmd_wan.WanImage(sprite_bin_decompressed)

        fragment: Fragment
        frame: Frame
        for mfg_i, frame in enumerate(image.meta_frame_store.meta_frame_groups):
            os.makedirs(f'/tmp/outimg/{s_i}/{mfg_i}', exist_ok=True)
            for mf_i, meta_frame_id in enumerate(frame.meta_frames_id):
                fragment = image.meta_frame_store.meta_frames[meta_frame_id]
                meta_frame_img_bytes: ImageBytes = image.image_store.images[fragment.image_index]
                resolution: Optional[FragmentResolution] = fragment.resolution
                try:
                    im = Image.frombuffer('RGBA',
                                          (resolution.x, resolution.y),
                                          bytearray(meta_frame_img_bytes.to_image(image.palette, fragment)),
                                          'raw', 'RGBA', 0, 1)

                    im.save(f'/tmp/outimg/{s_i}/{mfg_i}/{mf_i}.png')
                except ValueError as e:
                    print(f"Error converting {s_i}/{mfg_i}/{mf_i}", file=sys.stderr)
