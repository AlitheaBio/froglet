#!/usr/bin/env python3
"""Deterministic provider-readiness regression tests; no Docker or network."""
import importlib.util
import io
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import MagicMock, patch

SCRIPT = Path(__file__).resolve().parents[1] / "scripts/openshell-qualification-backend.py"
SPEC = importlib.util.spec_from_file_location("qualification_backend", SCRIPT)
BACKEND = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BACKEND)


class ProviderReadinessTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.log = Path(self.directory.name) / "node.log"
        self.log.write_text("Local Runtime API: http://127.0.0.1:8001\n")
        self.node = SimpleNamespace(log=self.log, process=MagicMock())
        self.node.process.poll.return_value = None

    def test_waits_for_delayed_provider_log_and_then_healthy_response(self):
        opener = MagicMock()
        opener.open.side_effect = [io.StringIO('{"status":"starting"}'),
                                   io.StringIO('{"status":"ok"}')]
        sleeps = []

        def advance(delay):
            sleeps.append(delay)
            if len(sleeps) == 1:
                with self.log.open("a") as output:
                    output.write("Local API Gateway: http://127.0.0.1:8002\n")

        with patch.object(BACKEND.time, "monotonic", side_effect=[0, 0, 1, 2]), \
             patch.object(BACKEND.time, "sleep", side_effect=advance), \
             patch.object(BACKEND.urllib.request, "build_opener", return_value=opener):
            self.assertEqual(BACKEND.wait_for_provider(self.node), "http://127.0.0.1:8002")
        self.assertEqual(len(sleeps), 2)
        self.assertEqual(opener.open.call_count, 2)
        opener.open.assert_called_with("http://127.0.0.1:8002/health", timeout=2)

    def test_missing_provider_listener_times_out(self):
        with patch.object(BACKEND.time, "monotonic", side_effect=[0, 0, 31]), \
             patch.object(BACKEND.time, "sleep"), \
             patch.object(BACKEND.urllib.request, "build_opener") as opener:
            with self.assertRaisesRegex(RuntimeError, "provider readiness timed out"):
                BACKEND.wait_for_provider(self.node)
        opener.return_value.open.assert_not_called()

    def test_node_exit_does_not_leave_readiness_wait_running(self):
        self.node.process.poll.return_value = 1
        with patch.object(BACKEND.time, "sleep") as sleep:
            with self.assertRaisesRegex(RuntimeError, "exited before provider readiness"):
                BACKEND.wait_for_provider(self.node)
        sleep.assert_not_called()


if __name__ == "__main__":
    unittest.main()
