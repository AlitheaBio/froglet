"""Failure and publication behavior of the FL example completeness record."""

from __future__ import annotations

from copy import deepcopy
from pathlib import Path
import sys
import unittest

REPO = Path(__file__).resolve().parents[2]
if str(REPO / "python") not in sys.path:
    sys.path.insert(0, str(REPO / "python"))

from froglet_fl_evidence import EvidenceError, RoundEvidence  # noqa: E402


class RoundEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.providers = {"silo_a": "a" * 64, "silo_b": "b" * 64}
        self.workloads = {"silo_a": "1" * 64, "silo_b": "2" * 64}
        self.context = {
            "run_id": "synthetic-reference-run",
            "round_id": 0,
            "operation": "map_shard_stats",
            "module_hash": "c" * 64,
        }
        self.ledger = RoundEvidence(
            **self.context, participants=self.providers, workload_hashes=self.workloads
        )

    def contribution(self, name: str) -> dict:
        marker = "3" if name == "silo_a" else "4"
        audit = {
            "offline_chain_verified": True,
            "artifact_count": 5,
            "deal_state": "succeeded",
            "execution_state": "succeeded",
            "provider_id": self.providers[name],
            "requester_id": "d" * 64,
            "descriptor_hash": "5" * 64,
            "offer_hash": "6" * 64,
            "quote_hash": "7" * 64,
            "deal_hash": "8" * 64,
            "receipt_hash": marker * 64,
            "result_hash": "9" * 64,
            "invoice_bundle_hash": None,
            "module_hash": self.context["module_hash"],
            "workload_hash": self.workloads[name],
        }
        return {
            **self.context,
            "workload_hash": self.workloads[name],
            "deal_id": f"deal-{name}",
            "verification": audit,
        }

    def stage_all(self) -> None:
        for name in self.providers:
            self.ledger.stage(name, **self.contribution(name))

    def test_missing_contribution_blocks_output_even_after_one_success(self) -> None:
        self.ledger.stage("silo_a", **self.contribution("silo_a"))
        with self.assertRaisesRegex(EvidenceError, "incomplete.*silo_b"):
            self.ledger.promote({"count": 20})
        snapshot = self.ledger.snapshot()
        self.assertEqual(snapshot["status"], "incomplete")
        self.assertEqual(snapshot["missing_participants"], ["silo_b"])
        self.assertIsNone(snapshot["output"])

    def test_failure_stays_visible_and_successful_retry_resolves_only_active_failure(self) -> None:
        self.ledger.record_failure("silo_b", "provider timeout")
        self.ledger.record_failure("silo_b", "provider timeout")
        self.ledger.record_failure("silo_b", "access refused")
        failed = self.ledger.snapshot()
        self.assertEqual(failed["failed_participants"], ["silo_b"])
        self.assertEqual(failed["failure_history"]["silo_b"], ["provider timeout", "access refused"])
        with self.assertRaisesRegex(EvidenceError, "incomplete"):
            self.ledger.promote({})
        self.stage_all()
        promoted = self.ledger.promote({"count": 20})
        self.assertEqual(promoted["failed_participants"], [])
        self.assertEqual(promoted["failure_history"], failed["failure_history"])

    def test_identical_retry_is_idempotent_before_and_after_promotion(self) -> None:
        args = self.contribution("silo_a")
        self.assertTrue(self.ledger.stage("silo_a", **args))
        self.assertFalse(self.ledger.stage("silo_a", **deepcopy(args)))
        self.ledger.stage("silo_b", **self.contribution("silo_b"))
        published = self.ledger.promote({"weights": [1, 2], "count": 20})
        self.assertFalse(self.ledger.stage("silo_a", **args))
        self.assertEqual(published, self.ledger.promote({"count": 20, "weights": [1, 2]}))
        self.assertEqual(len(published["contributions"]), 2)

    def test_conflicting_retry_preserves_the_accepted_contribution(self) -> None:
        args = self.contribution("silo_a")
        self.ledger.stage("silo_a", **args)
        before = self.ledger.snapshot()
        for field in ("receipt_hash", "result_hash", "deal_hash", "quote_hash"):
            changed = deepcopy(args)
            changed["verification"][field] = "e" * 64
            with self.subTest(field=field), self.assertRaisesRegex(EvidenceError, "conflicting"):
                self.ledger.stage("silo_a", **changed)
        self.assertEqual(before, self.ledger.snapshot())

    def test_cross_run_round_operation_and_module_are_rejected(self) -> None:
        changes = {"run_id": "other-run", "round_id": 1, "operation": "fl_local_sgd", "module_hash": "f" * 64}
        for field, value in changes.items():
            args = self.contribution("silo_a")
            args[field] = value
            with self.subTest(field=field), self.assertRaisesRegex(EvidenceError, "declared run/round/workload"):
                self.ledger.stage("silo_a", **args)
        self.assertEqual(self.ledger.snapshot()["contributions"], [])

    def test_relabeling_old_verified_workload_or_module_is_rejected(self) -> None:
        for field in ("workload_hash", "module_hash"):
            args = self.contribution("silo_a")
            args["verification"][field] = "f" * 64
            with self.subTest(field=field), self.assertRaisesRegex(EvidenceError, "verified .* hash differs"):
                self.ledger.stage("silo_a", **args)
        args = self.contribution("silo_a")
        args["workload_hash"] = "f" * 64
        args["verification"]["workload_hash"] = "f" * 64
        with self.assertRaisesRegex(EvidenceError, "declared request"):
            self.ledger.stage("silo_a", **args)

    def test_unexpected_participant_wrong_provider_and_reused_receipt_are_rejected(self) -> None:
        with self.assertRaisesRegex(EvidenceError, "unexpected participant"):
            self.ledger.stage("silo_c", **self.contribution("silo_a"))
        with self.assertRaisesRegex(EvidenceError, "unexpected participant"):
            self.ledger.record_failure("silo_c", "timeout")
        wrong_provider = self.contribution("silo_a")
        wrong_provider["verification"]["provider_id"] = self.providers["silo_b"]
        with self.assertRaisesRegex(EvidenceError, "provider differs"):
            self.ledger.stage("silo_a", **wrong_provider)
        self.ledger.stage("silo_a", **self.contribution("silo_a"))
        duplicate = self.contribution("silo_b")
        duplicate["verification"]["receipt_hash"] = "3" * 64
        with self.assertRaisesRegex(EvidenceError, "receipt is already assigned"):
            self.ledger.stage("silo_b", **duplicate)

    def test_declared_identity_and_workload_coverage_are_required(self) -> None:
        for providers, workloads in (
            ({}, {}),
            (self.providers, {"silo_a": "1" * 64}),
            (self.providers, {**self.workloads, "silo_c": "3" * 64}),
            ({"silo_a": "a" * 64, "silo_b": "a" * 64}, self.workloads),
        ):
            with self.subTest(providers=providers, workloads=workloads), self.assertRaises(EvidenceError):
                RoundEvidence(**self.context, participants=providers, workload_hashes=workloads)
        with self.assertRaisesRegex(EvidenceError, "round_id"):
            RoundEvidence(**{**self.context, "round_id": True}, participants=self.providers, workload_hashes=self.workloads)

    def test_unverified_malformed_or_inconsistent_audits_are_rejected(self) -> None:
        invalid_fields = (
            ("offline_chain_verified", False), ("offline_chain_verified", 1),
            ("deal_state", "canceled"), ("deal_state", "failed"),
            ("execution_state", "failed"), ("execution_state", None),
            ("receipt_hash", "ABC"), ("result_hash", None),
            ("artifact_count", 7), ("artifact_count", True),
            ("invoice_bundle_hash", "f" * 64),
        )
        for field, value in invalid_fields:
            args = self.contribution("silo_a")
            args["verification"][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(EvidenceError):
                self.ledger.stage("silo_a", **args)
        args = self.contribution("silo_a")
        args["verification"]["artifact_count"] = 6
        with self.assertRaisesRegex(EvidenceError, "invoice_bundle_hash"):
            self.ledger.stage("silo_a", **args)
        args["verification"]["invoice_bundle_hash"] = "f" * 64
        self.assertTrue(self.ledger.stage("silo_a", **args))

    def test_inputs_and_exported_snapshots_cannot_mutate_retained_evidence(self) -> None:
        args = self.contribution("silo_a")
        self.ledger.stage("silo_a", **args)
        self.providers["silo_a"] = "f" * 64
        self.workloads["silo_a"] = "f" * 64
        args["verification"]["result_hash"] = "f" * 64
        snapshot = self.ledger.snapshot()
        snapshot["participants"]["silo_a"] = "f" * 64
        snapshot["contributions"][0]["result_hash"] = "f" * 64
        self.assertEqual(self.ledger.snapshot()["participants"]["silo_a"], "a" * 64)
        self.assertEqual(self.ledger.snapshot()["contributions"][0]["result_hash"], "9" * 64)
        self.ledger.stage("silo_b", **self.contribution("silo_b"))
        output = {"nested": {"counts": [10, 20]}}
        published = self.ledger.promote(output)
        output["nested"]["counts"].append(30)
        published["output"]["nested"]["counts"].append(40)
        self.assertEqual(self.ledger.snapshot()["output"], {"nested": {"counts": [10, 20]}})

    def test_publication_cannot_be_replaced_by_changed_output_or_failure(self) -> None:
        self.stage_all()
        published = self.ledger.promote({"count": 20})
        with self.assertRaisesRegex(EvidenceError, "immutable"):
            self.ledger.promote({"count": 21})
        with self.assertRaisesRegex(EvidenceError, "immutable"):
            self.ledger.record_failure("silo_a", "late timeout")
        self.assertEqual(self.ledger.snapshot(), published)

    def test_failed_attempt_cannot_overwrite_staged_success(self) -> None:
        self.ledger.stage("silo_a", **self.contribution("silo_a"))
        with self.assertRaisesRegex(EvidenceError, "cannot replace"):
            self.ledger.record_failure("silo_a", "late failure")
        self.assertEqual(self.ledger.snapshot()["failure_history"], {})

    def test_invalid_output_is_not_promoted(self) -> None:
        self.stage_all()
        for output in ({"x": float("nan")}, {"x": float("inf")}, {1: "bad-key"}, {"x": (1, 2)}, []):
            with self.subTest(output=output), self.assertRaises(EvidenceError):
                self.ledger.promote(output)
        cyclic = {}
        cyclic["self"] = cyclic
        with self.assertRaises(EvidenceError):
            self.ledger.promote(cyclic)
        self.assertEqual(self.ledger.snapshot()["status"], "ready")
        self.assertIsNone(self.ledger.snapshot()["output"])

    def test_dataset_commitment_is_optional_declared_metadata(self) -> None:
        args = self.contribution("silo_a")
        self.ledger.stage("silo_a", **args, dataset_commitment="e" * 64)
        self.assertFalse(self.ledger.stage("silo_a", **args, dataset_commitment="sha256:" + "e" * 64))
        entry = self.ledger.snapshot()["contributions"][0]
        self.assertEqual(entry["dataset_commitment"], "sha256:" + "e" * 64)
        self.assertIn("dataset identity", self.ledger.snapshot()["evidence_boundary"])


if __name__ == "__main__":
    unittest.main()
