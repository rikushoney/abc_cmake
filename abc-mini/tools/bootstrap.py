#!/usr/bin/env python


import itertools
from collections.abc import Iterator, Sequence
from pathlib import Path
from typing import NamedTuple

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


class AbcModule(NamedTuple):
    name: str
    sources: list[str]


def walk_abc_srctree(abc_srcroot: Path) -> Iterator[AbcModule]:
    for dirpath, _, filenames in abc_srcroot.walk():
        if "module.make" in filenames:
            modname = dirpath.relative_to(abc_srcroot).as_posix()
            if modname in BLACKLISTED_MODULES:
                continue

            def blacklist(src: str) -> bool:
                return src not in BLACKLISTED_SOURCES

            srcs = list(filter(blacklist, parse_module_make(dirpath / "module.make")))
            if len(srcs) > 0:
                yield AbcModule(modname, srcs)


def sources_should_update(
    sources: Sequence[str],
    source_entries: Sequence[str],
) -> bool:
    return len(sources) != len(source_entries) or not all(
        c1 == c2 for c1, c2 in zip(sources, source_entries, strict=False)
    )


def main() -> int:
    script_root = Path(__file__).parent
    abc_mini_root = script_root.parent
    abc_root = abc_mini_root.parent
    abc_srcroot = abc_root / "src"
    modules = {mod.name: mod.sources for mod in walk_abc_srctree(abc_srcroot)}
    c_sources = []
    cpp_sources = []
    for src in itertools.chain.from_iterable(modules.values()):
        match Path(src).suffix:
            case ".c":
                c_sources.append(src)
            case ".cpp":
                cpp_sources.append(src)
            case _:
                print(f'unknown source "{src}"')
    c_sources.sort()
    cpp_sources.sort()
    abc_c_sources_txt = abc_mini_root / "abc_c_sources.txt"
    update_c_sources = True
    if abc_c_sources_txt.exists():
        c_source_entries = abc_c_sources_txt.read_text().split(";")
        update_c_sources = sources_should_update(c_sources, c_source_entries)
    abc_cpp_sources_txt = abc_mini_root / "abc_cpp_sources.txt"
    update_cpp_sources = True
    if abc_cpp_sources_txt.exists():
        cpp_source_entries = abc_cpp_sources_txt.read_text().split(";")
        update_cpp_sources = sources_should_update(cpp_sources, cpp_source_entries)
    if update_c_sources:
        print("updating c_sources.txt")
        abc_c_sources_txt.write_text(";".join(c_sources))
    if update_cpp_sources:
        print("updating cpp_sources.txt")
        abc_cpp_sources_txt.write_text(";".join(cpp_sources))
    if update_c_sources or update_cpp_sources:
        print("bumping CMakeLists.txt timestamp")
        (abc_mini_root / "CMakeLists.txt").touch()
    print("done!")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
