#!/usr/bin/env python

from pathlib import Path
from typing import Iterator


BLACKLISTED_MODULES: set[str] = {
    "map/fpga",
    "misc/espresso",
    "opt/fsim",
    "phys/place",
    "proof/int2",
    "sat/bsat2",
}

BLACKLISTED_SOURCES: set[str] = {"src/base/main/main.c"}


def parse_module_make(makefile: Path) -> Iterator[str]:
    lines = makefile.read_text().splitlines()
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if not line.startswith("SRC"):
            i += 1
            continue
        needle = "+="
        jump = line.find(needle) + len(needle)
        if jump < len(needle):
            raise ValueError(f'cannot parse line {i + 1}: "{line}"')
        line = line[jump:].strip()

        def clean(entry: str) -> str:
            return entry.rstrip("\\").strip()

        def isvalid(entry: str) -> bool:
            return len(entry) > 0

        entries = line.split()
        yield from filter(isvalid, map(clean, entries))
        while line.endswith("\\") and i + 1 < len(lines):
            i += 1
            line = lines[i].strip()
            entries = line.split()
            yield from filter(isvalid, map(clean, entries))
        i += 1


abc_srcroot = Path(__file__).parent / "src"
modules: dict[str, list[str]] = {}


for dirpath, _, filenames in abc_srcroot.walk():
    if "module.make" in filenames:
        modname = dirpath.relative_to(abc_srcroot).as_posix()
        if modname in BLACKLISTED_MODULES:
            continue

        def allowed(src: str) -> bool:
            return src not in BLACKLISTED_SOURCES

        sources = set(filter(allowed, parse_module_make(dirpath / "module.make")))
        if len(sources) > 0:
            modules[modname] = sorted(sources)


def base_modules() -> set[str]:
    return {m.split("/")[0] for m in modules}


libraries: dict[str, list[str]] = {mod: [] for mod in base_modules()}

for lib, srcs in libraries.items():
    submodules = [mod for mod in modules if mod.startswith(lib)]
    for mod in submodules:
        srcs += modules[mod]


def lib_entry(lib: str, srcs: list[str]) -> str:
    return f"Abc{lib.capitalize()}:" + ";".join(srcs)


libs_list = [lib_entry(lib, srcs) for lib, srcs in libraries.items()]
sources_txt = abc_srcroot.parent / "sources.txt"
sources_txt.write_text("".join(f"{entry}\n" for entry in sorted(libs_list)))
