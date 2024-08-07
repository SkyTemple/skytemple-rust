SkyTemple Rust Extensions
=========================

|build| |pypi-version| |pypi-downloads| |pypi-license| |pypi-pyversions| |discord|

.. |build| image:: https://img.shields.io/github/actions/workflow/status/SkyTemple/skytemple-rust/build-test-publish.yml
    :target: https://pypi.org/project/skytemple-rust/
    :alt: Build Status

.. |pypi-version| image:: https://img.shields.io/pypi/v/skytemple-rust
    :target: https://pypi.org/project/skytemple-rust/
    :alt: Version

.. |pypi-downloads| image:: https://img.shields.io/pypi/dm/skytemple-rust
    :target: https://pypi.org/project/skytemple-rust/
    :alt: Downloads

.. |pypi-license| image:: https://img.shields.io/pypi/l/skytemple-rust
    :alt: License (GPLv3)

.. |pypi-pyversions| image:: https://img.shields.io/pypi/pyversions/skytemple-rust
    :alt: Supported Python versions

.. |discord| image:: https://img.shields.io/discord/710190644152369162?label=Discord
    :target: https://discord.gg/4e3X36f
    :alt: Discord

Binary rust extensions for SkyTemple.

This implements a lot of file handlers for SkyTemple in Rust (prefixed ``st_``). You can read more
about the file types in the `SkyTemple Files`_ repository. This is also the main
place that these file handlers are used.

Additionally it has Python bindings for the following Rust crates:

- `pmd_wan`_ by marius851000_.

PLEASE NOTE that versions 1.3.4-1.3.x are intermediate releases. The only stable thing in it are the pmd_wan bindings!

Unit Tests
~~~~~~~~~~
Unit tests for the ``st_`` modules are located as Python Tests in `SkyTemple Files`_. The reason
for this is that they are tested together with the "legacy" Python implementations. When changing
existing modules, be aware that I will run the Python tests on them before merging any Pull Requests.

.. _SkyTemple Files: https://github.com/SkyTemple/skytemple-files
.. _pmd_wan: https://github.com/marius851000/pmd_wan
.. _marius851000: https://github.com/marius851000/
