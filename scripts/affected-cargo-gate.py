#!/usr/bin/env python3
"""Run Rust gates for changed Cargo packages and their reverse dependents.

Workspace-level manifests and toolchain configuration intentionally select the
whole workspace. A failed diff/metadata query also fails closed to the workspace.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


WORKSPACE_WIDE_FILES = {
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
    ".cargo/config",
    ".cargo/config.toml",
}


def run(command: list[str], cwd: Path, *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )


def package_graph(root: Path) -> tuple[dict[str, Path], dict[str, set[str]]]:
    metadata = json.loads(
        run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"], root, capture=True).stdout
    )
    paths: dict[str, Path] = {}
    reverse: dict[str, set[str]] = {}
    for package in metadata["packages"]:
        name = package["name"]
        paths[name] = Path(package["manifest_path"]).resolve().parent
        reverse.setdefault(name, set())
    names = set(paths)
    for package in metadata["packages"]:
        for dependency in package.get("dependencies", []):
            name = dependency["name"]
            if name in names:
                reverse.setdefault(name, set()).add(package["name"])
    return paths, reverse


def changed_paths(root: Path, base: str) -> list[str]:
    result = run(["git", "diff", "--name-only", "--diff-filter=ACDMRT", f"{base}...HEAD"], root, capture=True)
    return [line.replace("\\", "/") for line in result.stdout.splitlines() if line]


def affected_packages(root: Path, base: str, force_full: bool = False) -> tuple[list[str], str]:
    if force_full:
        return [], "workspace"
    packages, reverse = package_graph(root)
    changed = changed_paths(root, base)
    if any(path in WORKSPACE_WIDE_FILES or path.startswith(".cargo/") for path in changed):
        return [], "workspace"

    owners: set[str] = set()
    package_paths = sorted(packages.items(), key=lambda item: len(item[1].parts), reverse=True)
    for changed_path in changed:
        absolute = (root / changed_path).resolve()
        for package_name, package_path in package_paths:
            if absolute == package_path or package_path in absolute.parents:
                owners.add(package_name)
                break
        else:
            # Rust build scripts or workspace-level code outside a member package
            # may affect every build, so unknown source changes fail closed.
            if Path(changed_path).suffix in {".rs", ".toml"}:
                return [], "workspace"

    if not owners:
        return [], "skip"

    affected = set(owners)
    pending = list(owners)
    while pending:
        current = pending.pop()
        for dependent in reverse.get(current, set()):
            if dependent not in affected:
                affected.add(dependent)
                pending.append(dependent)
    return sorted(affected), "packages"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True, help="merge-base commit or ref")
    parser.add_argument("--gate", choices=("clippy", "test"), required=True)
    parser.add_argument("--force-full", action="store_true")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    try:
        selected, mode = affected_packages(root, args.base, args.force_full)
    except (subprocess.CalledProcessError, KeyError, json.JSONDecodeError) as error:
        print(f"Affected-package discovery failed ({error}); running the full Rust workspace gate.", file=sys.stderr)
        selected, mode = [], "workspace"

    if mode == "skip":
        print(f"No Cargo package changes detected; skipping Rust {args.gate} gate.")
        return 0

    print(f"Rust {args.gate} scope: {mode}" + (f" ({', '.join(selected)})" if selected else ""), flush=True)
    command = ["cargo", args.gate]
    if mode == "workspace":
        command.append("--workspace")
    else:
        for package in selected:
            command.extend(("--package", package))
    if args.gate == "clippy":
        command.extend(("--all-targets", "--", "-D", "warnings"))
    run(command, root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
