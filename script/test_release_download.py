#!/usr/bin/env python3
"""Exercise the release downloader with local fixtures; never download or install."""
import hashlib
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "script/install_from_release.sh").read_text()


class ReleaseDownloadTests(unittest.TestCase):
    def setUp(self):
        (ROOT / "target").mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=ROOT / "target")
        self.addCleanup(self.temp.cleanup)
        self.work = Path(self.temp.name)
        self.fixture = self.work / "fixture"
        self.fixture.mkdir()
        self.downloads = self.work / "downloads"
        self.downloads.mkdir()
        self.record = self.work / "executed"
        self.name = "Kaku2Okur-install-unsigned-" + os.uname().machine + ".sh"
        payload = b'#!/usr/bin/env bash\nprintf "%s\\n" "$@" > "$TEST_RESULT"\n'
        (self.fixture / self.name).write_bytes(payload)
        self.digest = hashlib.sha256(payload).hexdigest()
        self.manifest = self.fixture / (self.name + ".sha256")
        self.manifest.write_text(self.digest + "  " + self.name + "\n")
        fake = self.work / "fake-curl"
        fake.write_text(
            "#!" + sys.executable + "\n"
            "import os, pathlib, shutil, sys\n"
            "from urllib.parse import urlparse\n"
            "if os.environ.get('TEST_NETWORK_FAILURE'): sys.exit(22)\n"
            "args = sys.argv[1:]\n"
            "assert args[0] == '--disable'\n"
            "assert args[args.index('--proto')+1] == '=https'\n"
            "assert args[args.index('--proto-redir')+1] == '=https'\n"
            "url = args[-1]\n"
            "assert url.startswith('https://github.com/yakitori-daisuki/Kaku2Okur/releases/latest/download/')\n"
            "shutil.copyfile(pathlib.Path(os.environ['TEST_FIXTURE']) / pathlib.Path(urlparse(url).path).name, args[args.index('--output')+1])\n"
        )
        fake.chmod(0o755)
        self.runner = self.work / "download.sh"
        self.runner.write_text(SOURCE.replace("/usr/bin/curl", '"' + str(fake) + '"'))
        self.env = dict(os.environ, TEST_FIXTURE=str(self.fixture),
                        TEST_RESULT=str(self.record), TMPDIR=str(self.downloads))

    def run_download(self, args=("--verify-only",)):
        result = subprocess.run(["bash", str(self.runner), *args],
                                env=self.env, text=True, capture_output=True)
        self.assertEqual(list(self.downloads.iterdir()), [], "temporary files leaked")
        return result

    def assert_rejected(self):
        result = self.run_download()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.record.exists(), "unverified installer executed")

    def test_verified_payload_executes_in_verify_mode(self):
        self.assertEqual(self.run_download().returncode, 0)
        self.assertIn("--verify-only", self.record.read_text().splitlines())

    def test_checksum_mismatch_never_executes(self):
        self.manifest.write_text("0" * 64 + "  " + self.name + "\n")
        self.assert_rejected()

    def test_wrong_filename_never_executes(self):
        self.manifest.write_text(self.digest + "  other.sh\n")
        self.assert_rejected()

    def test_extra_manifest_line_never_executes(self):
        self.manifest.write_text(self.manifest.read_text() * 2)
        self.assert_rejected()

    def test_extra_manifest_field_never_executes(self):
        self.manifest.write_text(self.digest + "  " + self.name + " extra\n")
        self.assert_rejected()

    def test_failed_download_never_executes(self):
        self.env["TEST_NETWORK_FAILURE"] = "1"
        self.assert_rejected()

    def test_explicit_mode_required(self):
        self.assertEqual(self.run_download(args=()).returncode, 2)
        self.assertFalse(self.record.exists())

    def test_readme_commands_match_script(self):
        expected = "\n".join(SOURCE.splitlines()[2:])
        for name in ("README.md", "README.ja.md"):
            body = (ROOT / name).read_text()
            match = re.search(r"/bin/bash -c '\n(.*?)\n' -- --allow-unsigned", body, re.S)
            self.assertIsNotNone(match, name)
            self.assertEqual(match[1], expected, name)
            subprocess.run(["bash", "-n"], input=match[0], text=True, check=True)


if __name__ == "__main__":
    unittest.main()
