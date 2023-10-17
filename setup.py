from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="skytemple-rust",
    rust_extensions=[RustExtension(f"skytemple_rust", binding=Binding.PyO3)],
    packages=["skytemple_rust"],
    zip_safe=False,
)
