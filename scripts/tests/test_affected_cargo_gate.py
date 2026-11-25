import subprocess
import tempfile
import unittest
import importlib.util
import contextlib
import io
from pathlib import Path
from unittest import mock

GATE_SPEC = importlib.util.spec_from_file_location(
    "affected_cargo_gate", Path(__file__).resolve().parents[1] / "affected-cargo-gate.py"
)
GATE_MODULE = importlib.util.module_from_spec(GATE_SPEC)
GATE_SPEC.loader.exec_module(GATE_MODULE)
affected_packages = GATE_MODULE.affected_packages


class AffectedCargoGateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "crates/base/src").mkdir(parents=True)
        (self.root / "crates/app/src").mkdir(parents=True)
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nresolver = "2"\nmembers = ["crates/base", "crates/app"]\n',
            encoding="utf-8",
        )
        (self.root / "crates/base/Cargo.toml").write_text(
            '[package]\nname = "base-crate"\nversion = "0.1.0"\nedition = "2021"\n',
            encoding="utf-8",
        )
        (self.root / "crates/app/Cargo.toml").write_text(
            '[package]\nname = "app-crate"\nversion = "0.1.0"\nedition = "2021"\n'
            '[dependencies]\nbase-crate = { path = "../base" }\n',
            encoding="utf-8",
        )
        (self.root / "crates/base/src/lib.rs").write_text("pub fn value() -> u8 { 1 }\n", encoding="utf-8")
        (self.root / "crates/app/src/lib.rs").write_text(
            "pub fn value() -> u8 { base_crate::value() }\n", encoding="utf-8"
        )
        self.git("init", "-q")
        self.git("config", "user.email", "ci@example.invalid")
        self.git("config", "user.name", "CI fixture")
        self.git("add", ".")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD").stdout.strip()

    def tearDown(self):
        self.temp.cleanup()

    def git(self, *args):
        return subprocess.run(
            ["git", *args], cwd=self.root, check=True, text=True, capture_output=True
        )

    def test_source_change_selects_reverse_dependents(self):
        source = self.root / "crates/base/src/lib.rs"
        source.write_text("pub fn value() -> u8 { 2 }\n", encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "-qm", "change base source")
        selected, mode = affected_packages(self.root, self.base)
        self.assertEqual(mode, "packages")
        self.assertEqual(selected, ["app-crate", "base-crate"])

    def test_workspace_manifest_change_selects_every_package(self):
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nresolver = "2"\nmembers = ["crates/base", "crates/app"]\n\n'
            '[workspace.package]\nedition = "2021"\n',
            encoding="utf-8",
        )
        self.git("add", ".")
        self.git("commit", "-qm", "change workspace manifest")
        selected, mode = affected_packages(self.root, self.base)
        self.assertEqual(mode, "workspace")
        self.assertEqual(selected, [])

    def test_non_rust_change_skips_cargo_gates(self):
        (self.root / "README.md").write_text("docs only\n", encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "-qm", "change docs")
        selected, mode = affected_packages(self.root, self.base)
        self.assertEqual(mode, "skip")
        self.assertEqual(selected, [])

    def test_unowned_rust_source_fails_closed_to_workspace(self):
        unowned = self.root / "build.rs"
        unowned.write_text("fn main() {}\n", encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "-qm", "change unowned build script")
        selected, mode = affected_packages(self.root, self.base)
        self.assertEqual(mode, "workspace")
        self.assertEqual(selected, [])

    def test_force_full_selects_workspace_without_discovery(self):
        with mock.patch.object(GATE_MODULE, "package_graph", side_effect=AssertionError("must not inspect")):
            selected, mode = affected_packages(self.root, self.base, force_full=True)
        self.assertEqual(mode, "workspace")
        self.assertEqual(selected, [])

    def test_discovery_failure_runs_full_workspace_gate(self):
        error = subprocess.CalledProcessError(1, ["git", "diff"])
        stderr = io.StringIO()
        stdout = io.StringIO()
        with (
            mock.patch.object(GATE_MODULE, "affected_packages", side_effect=error),
            mock.patch.object(GATE_MODULE, "run") as run_gate,
            mock.patch.object(GATE_MODULE.sys, "argv", ["affected-cargo-gate.py", "--base", self.base, "--gate", "test"]),
            contextlib.redirect_stderr(stderr),
            contextlib.redirect_stdout(stdout),
        ):
            result = GATE_MODULE.main()

        self.assertEqual(result, 0)
        run_gate.assert_called_once_with(
            ["cargo", "test", "--workspace"],
            Path(GATE_MODULE.__file__).resolve().parents[1],
        )
        self.assertIn("running the full Rust workspace gate", stderr.getvalue())
        self.assertIn("Rust test scope: workspace", stdout.getvalue())


if __name__ == "__main__":
    unittest.main()
