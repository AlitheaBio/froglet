"""Reference semantics and failure gates; live provider checks are in the CLI.

The optional FLARE test uses installed components with a stub worker; the live
CLI always requires FLARE and executes real signed provider-local Wasm deals.
"""
import copy
from contextlib import closing
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import sqlite3
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

from examples import immunopeptidomics_demo as demo
from python.froglet_fl_evidence import EvidenceError, RoundEvidence


class ScientificOracleTests(unittest.TestCase):
    def setUp(self):
        self.fixture = demo.load_fixture(demo.FIXTURE)
        self.query = self.fixture["query"]
        self.site = self.fixture["sites"]["site_a"]

    def test_hand_counted_fixture_and_weighted_denominator(self):
        a = demo.oracle(self.site, self.query)
        b = demo.oracle(self.fixture["sites"]["site_b"], self.query)
        self.assertEqual(a["counts"], dict(zip(demo.COUNT_FIELDS, [8, 2, 1, 5, 3, 2, 1, 1, 1, 2])))
        self.assertEqual(b["counts"], dict(zip(demo.COUNT_FIELDS, [8, 1, 1, 6, 5, 3, 2, 1, 0, 1])))
        result = demo.aggregate_results([a, b], self.query)
        self.assertEqual(result["counts"], dict(zip(demo.COUNT_FIELDS, [16, 3, 2, 11, 8, 5, 3, 2, 1, 3])))
        self.assertEqual(demo.detection_fraction(result), {"numerator": 5, "denominator": 8, "value": 0.625})
        self.assertNotEqual(0.625, (2 / 3 + 3 / 5) / 2)

    def test_duplicate_detections_and_hla_rows_do_not_multiply_samples(self):
        changed = copy.deepcopy(self.site)
        changed["sample_peptide_detections"] *= 3
        changed["sample_hla_typing"] *= 2
        self.assertEqual(demo.oracle(changed, self.query), demo.oracle(self.site, self.query))

    def test_incomplete_nonmatching_hla_is_unknown_and_recorded_target_is_eligible(self):
        # A3 has the target with incomplete typing. A7 lacks it with incomplete
        # typing, whereas A6 lacks it with complete typing at the requested locus.
        baseline = demo.oracle(self.site, self.query)["counts"]
        changed = copy.deepcopy(self.site)
        for row in changed["sample_hla_typing_status"]:
            if row["sample_id"] == "A7":
                row["status"] = "complete"
            elif row["sample_id"] == "A3":
                row["status"] = "missing"
        actual = demo.oracle(changed, self.query)["counts"]
        self.assertEqual(actual["eligible"], baseline["eligible"])
        self.assertEqual(actual["detected"], baseline["detected"])
        self.assertEqual(actual["hla_unknown"], baseline["hla_unknown"] - 1)
        self.assertEqual(actual["hla_nonmatching"], baseline["hla_nonmatching"] + 1)

    def test_missing_typing_row_is_unknown_at_requested_locus(self):
        changed = copy.deepcopy(self.site)
        changed["sample_hla_typing_status"] = [row for row in changed["sample_hla_typing_status"]
                                               if row["sample_id"] != "A6"]
        changed["sample_hla_typing_status"].append({"sample_id": "A6", "locus": "HLA-B", "status": "complete"})
        actual = demo.oracle(changed, self.query)["counts"]
        self.assertEqual(actual["hla_unknown"], 3)
        self.assertEqual(actual["hla_nonmatching"], 0)

    def test_unmeasured_and_failed_observations_never_become_detected(self):
        changed = copy.deepcopy(self.site)
        for sample in ("A4", "A5"):
            changed["sample_peptide_detections"].append({"sample_id": sample, "sequence": self.query["candidate_peptide"]})
        self.assertEqual(demo.oracle(changed, self.query), demo.oracle(self.site, self.query))
        for row in changed["samples"]:
            if row["assay_status"] == "measured":
                row["assay_status"] = "not_measured"
        result = demo.oracle(changed, self.query)
        self.assertEqual(result["counts"]["detected"], 0)
        self.assertEqual(demo.detection_fraction(result), {"numerator": 0, "denominator": 0, "value": None})

    def test_empty_cohort_and_exact_peptide_matching(self):
        query = dict(self.query, organ="Synthetic_organ_a")
        self.assertEqual(set(demo.oracle(self.site, query)["counts"].values()), {0})
        query = dict(self.query, candidate_peptide="ACDEFGHIL")
        result = demo.oracle(self.site, query)
        self.assertEqual(result["counts"]["measured"], 3)
        self.assertEqual(result["counts"]["detected"], 0)
        self.assertEqual(result["counts"]["not_detected"], 3)

    def test_malformed_inputs_and_duplicate_samples_rejected(self):
        for query in ({}, dict(self.query, op="extra"), dict(self.query, hla_allele="A0201"),
                      dict(self.query, hla_allele="HLA-A*02:01:01"),
                      dict(self.query, candidate_peptide="ACDEFGHIK[mod]"), dict(self.query, organ=" ")):
            with self.subTest(query=query), self.assertRaises(RuntimeError):
                demo.oracle(self.site, query)
        for table in ("samples", "sample_hla_typing_status"):
            changed = copy.deepcopy(self.site)
            changed[table].append(dict(changed[table][0]))
            with self.subTest(table=table), self.assertRaises(RuntimeError):
                demo.oracle(changed, self.query)

    def test_result_rejects_denominator_corruption_and_sample_level_fields(self):
        for mutate in (lambda result: result["counts"].update(measured=99),
                       lambda result: result.update(sample_ids=["A1"]),
                       lambda result: result["counts"].update(detected=True),
                       lambda result: result.update(interpretation={})):
            result = demo.oracle(self.site, self.query)
            mutate(result)
            with self.assertRaises(RuntimeError):
                demo.validate_result(result, self.query)

    def test_sqlite_contains_only_own_site_rows_and_read_policy(self):
        with tempfile.TemporaryDirectory() as folder:
            node = SimpleNamespace(data=Path(folder), env={})
            database = demo.provision_sqlite(node, self.site)
            with closing(sqlite3.connect(database)) as connection:
                ids = {row[0] for row in connection.execute("SELECT sample_id FROM samples")}
                self.assertEqual(ids, {"A" + str(i) for i in range(1, 11)})
                self.assertEqual(connection.execute("SELECT COUNT(*) FROM sample_peptide_detections WHERE sample_id='A1' AND sequence='ACDEFGHIK'").fetchone()[0], 2)
            policy = Path(node.env["FROGLET_WASM_POLICY_PATH"]).read_text()
            self.assertIn("[sqlite.handles.cohort]", policy)
            self.assertIn("max_queries_per_execution = 1", policy)
            self.assertNotIn("[http]", policy)

    def test_submission_commits_query_and_module_without_fixture_rows(self):
        module = b"\0asm\x01\0\0\0"
        submission = demo.build_submission(module, self.query)
        self.assertEqual(submission["input"], self.query)
        self.assertEqual(submission["workload"]["requested_capabilities"], [demo.CAPABILITY])
        changed = demo.build_submission(module, dict(self.query, organ="synthetic_organ_b"))
        self.assertNotEqual(demo.froglet_verify.payload_hash(submission["workload"]),
                            demo.froglet_verify.payload_hash(changed["workload"]))
        self.assertNotIn("samples", json.dumps(submission))


class OfflineEvidenceTests(unittest.TestCase):
    def setUp(self):
        fixture = json.loads((demo.REPO / "conformance/kernel_v1.json").read_text())
        self.artifacts = {name: copy.deepcopy(fixture["artifacts"]["free_" + name if name != "descriptor" else name]["artifact"])
                          for name in ("descriptor", "offer", "quote", "deal", "receipt")}
        self.result = fixture["result"]
        self.evidence = {"signed_artifacts": self.artifacts, "deal_id": self.artifacts["deal"]["hash"],
                         "result_hash": self.artifacts["receipt"]["payload"]["result_hash"]}

    def test_real_signed_free_conformance_chain_verifies(self):
        audit = demo.verify_artifact_chain_offline(self.evidence, self.result)
        self.assertTrue(audit["offline_chain_verified"])
        self.assertEqual(audit["artifact_count"], 5)

    def test_tampered_signature_and_result_fail(self):
        changed = copy.deepcopy(self.evidence)
        signature = changed["signed_artifacts"]["receipt"]["signature"]
        changed["signed_artifacts"]["receipt"]["signature"] = ("1" if signature[0] == "0" else "0") + signature[1:]
        with self.assertRaisesRegex(RuntimeError, "verification failed"):
            demo.verify_artifact_chain_offline(changed, self.result)
        with self.assertRaisesRegex(RuntimeError, "Result differs"):
            demo.verify_artifact_chain_offline(self.evidence, {"tampered": True})

    def test_missing_participant_blocks_promotion_and_retry_is_idempotent(self):
        audit = demo.verify_artifact_chain_offline(self.evidence, self.result)
        module_hash = audit["module_hash"]
        workload = audit["workload_hash"]
        evidence = RoundEvidence(run_id="reference-test", round_id=0, operation=demo.OPERATION,
            module_hash=module_hash, participants={"site_a": audit["provider_id"], "site_b": "f" * 64},
            workload_hashes={"site_a": workload, "site_b": workload})
        record = dict(self.evidence, result=self.result)
        demo.stage_records(evidence, {"site_a": record}, {"site_b": {"type": "ConnectionError", "message": "unavailable"}},
            run_id="reference-test", module_hash=module_hash,
            workload_hashes={"site_a": workload, "site_b": workload}, fixture_sites={"site_a": {}})
        snapshot = evidence.snapshot()
        self.assertEqual(len(snapshot["contributions"]), 1)
        self.assertEqual(snapshot["missing_participants"], ["site_b"])
        self.assertIsNone(snapshot["output"])
        with self.assertRaises(EvidenceError):
            evidence.promote({"counts": "incomplete"})

    def test_atomic_report_and_cli_dependency_failure(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "report.json"
            demo.write_report(path, {"status": "succeeded"})
            demo.write_report(path, {"status": "failed", "aggregate": None})
            self.assertEqual(json.loads(path.read_text()), {"status": "failed", "aggregate": None})
            self.assertEqual(list(path.parent.iterdir()), [path])
        with mock.patch.object(demo.importlib.metadata, "version", return_value="999"):
            if importlib.util.find_spec("nvflare"):
                with self.assertRaisesRegex(RuntimeError, "2.9.0"):
                    demo.nvflare_components()

    def test_cli_without_framework_dependency_fails_explicitly(self):
        # -S excludes installed site-packages while allowing this reference's
        # zero-dependency verifier and committed runtime helper to import.
        result = subprocess.run([sys.executable, "-I", "-S", "-B", str(Path(demo.__file__)), "--no-build"],
                                capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 1)
        self.assertIn("NVIDIA FLARE is required", result.stderr)
        self.assertNotIn("Traceback", result.stderr)


@unittest.skipUnless(importlib.util.find_spec("nvflare"), "NVIDIA FLARE not installed; live CLI requires it")
class InstalledFlareTests(unittest.TestCase):
    def test_two_executor_dxo_paths_and_site_failure_are_observable(self):
        fixture = demo.load_fixture(demo.FIXTURE)
        seen = []
        def worker(name, query):
            seen.append(name)
            if name == "site_b":
                raise ConnectionError("synthetic unavailable worker")
            return demo.oracle(fixture["sites"][name], query)
        records, failures = demo.dispatch_nvflare(sorted(fixture["sites"]), fixture["query"], worker)
        self.assertEqual(set(seen), {"site_a", "site_b"})
        self.assertEqual(set(records), {"site_a"})
        self.assertEqual(failures["site_b"]["type"], "ConnectionError")
        self.assertEqual(records["site_a"]["counts"]["detected"], 2)


if __name__ == "__main__":
    unittest.main()
