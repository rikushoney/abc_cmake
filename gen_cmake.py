#!/usr/bin/env python

import sys

assert sys.version_info.major >= 3, "only Python >= 3 is supported"

abc_modules = [
    "src/base/abc",
    "src/base/abci",
    "src/base/cmd",
    "src/base/io",
    "src/base/main",
    "src/base/exor",
    "src/base/ver",
    "src/base/wlc",
    "src/base/wln",
    "src/base/acb",
    "src/base/bac",
    "src/base/cba",
    "src/base/pla",
    # "src/base/test",
    "src/map/mapper",
    "src/map/mio",
    "src/map/super",
    "src/map/if",
    "src/map/amap",
    "src/map/cov",
    "src/map/scl",
    "src/map/mpm",
    "src/misc/extra",
    "src/misc/mvc",
    "src/misc/st",
    "src/misc/util",
    "src/misc/nm",
    "src/misc/vec",
    "src/misc/hash",
    "src/misc/tim",
    "src/misc/bzlib",
    "src/misc/zlib",
    "src/misc/mem",
    "src/misc/bar",
    "src/misc/bbl",
    "src/misc/parse",
    "src/opt/cut",
    "src/opt/fxu",
    "src/opt/fxch",
    "src/opt/rwr",
    "src/opt/mfs",
    "src/opt/sim",
    "src/opt/ret",
    "src/opt/fret",
    "src/opt/res",
    "src/opt/lpk",
    "src/opt/nwk",
    "src/opt/rwt",
    "src/opt/cgt",
    "src/opt/csw",
    "src/opt/dar",
    "src/opt/dau",
    "src/opt/dsc",
    "src/opt/sfm",
    "src/opt/sbd",
    "src/sat/bsat",
    "src/sat/xsat",
    "src/sat/satoko",
    "src/sat/csat",
    "src/sat/msat",
    "src/sat/psat",
    "src/sat/cnf",
    "src/sat/bmc",
    "src/sat/glucose",
    "src/sat/glucose2",
    "src/bool/bdc",
    "src/bool/deco",
    "src/bool/dec",
    "src/bool/kit",
    "src/bool/lucky",
    "src/bool/rsb",
    "src/bool/rpo",
    "src/proof/pdr",
    "src/proof/abs",
    "src/proof/live",
    "src/proof/ssc",
    "src/proof/int",
    "src/proof/cec",
    "src/proof/acec",
    "src/proof/dch",
    "src/proof/fraig",
    "src/proof/fra",
    "src/proof/ssw",
    "src/aig/aig",
    "src/aig/saig",
    "src/aig/gia",
    "src/aig/ioa",
    "src/aig/ivy",
    "src/aig/hop",
    "src/aig/miniaig",
]


def read_file(filename, mode="r"):
    with open(filename, mode) as f:
        return f.read()


def extract_srcs(contents):
    contents = contents.strip()
    if not contents.startswith("SRC"):
        return list()
    contents = contents.removeprefix("SRC").strip()
    if not contents.startswith("+="):
        return list()
    contents = contents.removeprefix("+=").strip()
    return [src.strip().removesuffix("\\").strip() for src in contents.splitlines()]


def write_module_cmakelists(dir, srcs):
    if len(srcs) == 0:
        return
    with open(f"{dir}/CMakeLists.txt", "w") as f:
        print("target_sources(abc\n  PRIVATE", file=f)
        for src in srcs:
            print(f"  {src}", file=f)
        print(")", file=f)


def write_dir_cmakelists(dir, mods):
    with open(f"{dir}/CMakeLists.txt", "w") as f:
        for mod in mods:
            print(f"add_subdirectory({mod})", file=f)


def main():
    abc_srcs = [
        {"module": module, "srcs": extract_srcs(read_file(f"{module}/module.make"))}
        for module in abc_modules
    ]
    base_dirs = dict()
    for mod in abc_srcs:
        mod_name = mod["module"].split("/")[-1]
        print(f"found module {mod_name}")
        if "phys" in mod_name:
            print(mod)
        if len(mod["srcs"]) == 0:
            print(f"module {mod_name} has no sources")
            continue
        base_dir = "/".join(mod["module"].split("/")[:-1])
        if base_dirs.get(base_dir) is None:
            base_dirs[base_dir] = list()
        base_dirs[base_dir].append(mod_name)
        write_module_cmakelists(mod["module"], mod["srcs"])
    for dir, mods in base_dirs.items():
        write_dir_cmakelists(dir, mods)


if __name__ == "__main__":
    main()
