import os

import toml as toml
from setuptools import setup
from setuptools_rust import Binding, RustExtension

with open(os.path.join(os.path.dirname(__file__), 'Cargo.toml'), 'r') as f:
    cargo_toml = toml.load(f)

# README read-in
from os import path
this_directory = path.abspath(path.dirname(__file__))
with open(path.join(this_directory, 'README.rst'), encoding='utf-8') as f:
    long_description = f.read()
# END README read-in

setup(
    name="skytemple-rust",
    version=cargo_toml['package']['version'],
    rust_extensions=[RustExtension("skytemple_rust.pmd_wan", binding=Binding.PyO3)],
    packages=["skytemple_rust"],
    description='Binary Rust extension for skytemple-files',
    long_description=long_description,
    long_description_content_type='text/x-rst',
    url='https://github.com/SkyTemple/skytemple-rust/',
    classifiers=[
        'Development Status :: 3 - Alpha',
        'Programming Language :: Python',
        'Programming Language :: Rust',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Python :: 3.8'
    ],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
    include_package_data=True,
)