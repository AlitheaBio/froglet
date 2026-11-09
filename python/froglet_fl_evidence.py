"""Completeness records for the federated-learning reference example.

This application helper consumes the output of the example's existing offline
artifact verifier. It does not verify signatures, authenticate datasets, prove
correct computation, or change Froglet protocol/settlement semantics. Callers
must extract workload hashes from the verified Quote and declare expected
workloads before collecting contributions. Run/round labels are application
metadata, not additional signed protocol fields.

The record is in-memory. Exporting a snapshot does not implement durable
recovery or a distributed commit protocol.
"""

from __future__ import annotations

from collections.abc import Mapping
import json
import re
from typing import Any


class EvidenceError(ValueError):
    """A contribution or publication conflicts with the declared example run."""


def _text(value: object, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise EvidenceError(f"{field} must be a non-empty string")
    return value


def _hash(value: object, field: str) -> str:
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise EvidenceError(f"{field} must be a 64-character lowercase hexadecimal string")
    return value


def _round(value: object) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise EvidenceError("round_id must be a non-negative integer")
    return value


def _json_copy_bytes(value: object) -> str:
    """Freeze JSON data without accepting lossy key/type coercions or NaN."""
    def check(item: object) -> None:
        if item is None or type(item) in (str, bool, int, float):
            return
        if type(item) is list:
            for child in item:
                check(child)
            return
        if type(item) is dict:
            for key, child in item.items():
                if not isinstance(key, str):
                    raise EvidenceError("output object keys must be strings")
                check(child)
            return
        raise EvidenceError("output must contain only JSON values")

    try:
        check(value)
        return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)
    except (ValueError, TypeError, RecursionError) as error:
        raise EvidenceError(f"output is not finite JSON data: {error}") from error


class RoundEvidence:
    """Stage one verified contribution per declared participant before publication.

    ``workload_hashes`` binds each participant to its expected request, including
    the input/model state committed by that request. Reusing the same expected
    hashes across rounds does not create a new cryptographic round binding.
    An identical retry is harmless; replacing accepted evidence is forbidden.
    A successful retry can resolve a recorded failure without erasing its reason.
    """

    def __init__(
        self,
        *,
        run_id: str,
        round_id: int,
        operation: str,
        module_hash: str,
        participants: Mapping[str, str],
        workload_hashes: Mapping[str, str],
    ) -> None:
        self._run_id = _text(run_id, "run_id")
        self._round_id = _round(round_id)
        self._operation = _text(operation, "operation")
        self._module_hash = _hash(module_hash, "module_hash")
        if not isinstance(participants, Mapping) or not participants:
            raise EvidenceError("participants must be a non-empty mapping")
        self._participants = {
            _text(name, "participant_id"): _hash(provider, "provider_id")
            for name, provider in participants.items()
        }
        if len(set(self._participants.values())) != len(self._participants):
            raise EvidenceError("participants must have distinct provider identities")
        if not isinstance(workload_hashes, Mapping) or set(workload_hashes) != set(
            self._participants
        ):
            raise EvidenceError("workload_hashes must cover exactly the declared participants")
        self._workload_hashes = {
            name: _hash(workload_hashes[name], "workload_hash")
            for name in self._participants
        }
        self._contributions: dict[str, str] = {}
        self._failures: dict[str, list[str]] = {}
        self._output: str | None = None

    def _participant(self, participant_id: str) -> None:
        if not isinstance(participant_id, str) or participant_id not in self._participants:
            raise EvidenceError("unexpected participant")

    def stage(
        self,
        participant_id: str,
        *,
        run_id: str,
        round_id: int,
        operation: str,
        module_hash: str,
        workload_hash: str,
        deal_id: str,
        verification: Mapping[str, Any],
        dataset_commitment: str | None = None,
    ) -> bool:
        """Stage already-verified evidence; return False for an identical retry.

        ``verification`` must come from ``verify_artifact_chain_offline``, not
        from an untrusted network response asserting that verification passed.
        The verifier must return ``workload_hash`` from the verified Quote and
        ``module_hash`` from the verified Receipt executor. This helper compares
        those values to the declaration; it does not rerun the cryptography.
        """
        self._participant(participant_id)
        if (
            _text(run_id, "run_id") != self._run_id
            or _round(round_id) != self._round_id
            or _text(operation, "operation") != self._operation
            or _hash(module_hash, "module_hash") != self._module_hash
        ):
            raise EvidenceError("contribution does not match the declared run/round/workload")
        if _hash(workload_hash, "workload_hash") != self._workload_hashes[participant_id]:
            raise EvidenceError("contribution workload hash differs from the declared request")
        if not isinstance(verification, Mapping) or verification.get("offline_chain_verified") is not True:
            raise EvidenceError("offline artifact verification is required")
        if verification.get("deal_state") != "succeeded" or verification.get("execution_state") != "succeeded":
            raise EvidenceError("a verified successful deal and execution are required")
        if _hash(verification.get("workload_hash"), "verified workload_hash") != workload_hash:
            raise EvidenceError("verified workload hash differs from the declared request")
        if _hash(verification.get("module_hash"), "verified module_hash") != self._module_hash:
            raise EvidenceError("verified module hash differs from the declared module")
        provider_id = _hash(verification.get("provider_id"), "verified provider_id")
        if provider_id != self._participants[participant_id]:
            raise EvidenceError("verified provider differs from the declared participant")
        artifact_count = verification.get("artifact_count")
        if type(artifact_count) is not int or artifact_count not in (5, 6):
            raise EvidenceError("verified artifact_count must be 5 or 6")
        hashes = {
            name: _hash(verification.get(name), f"verified {name}")
            for name in (
                "descriptor_hash", "offer_hash", "quote_hash", "deal_hash",
                "receipt_hash", "result_hash", "requester_id",
            )
        }
        invoice_hash = verification.get("invoice_bundle_hash")
        if artifact_count == 6:
            invoice_hash = _hash(invoice_hash, "verified invoice_bundle_hash")
        elif invoice_hash is not None:
            raise EvidenceError("five-artifact evidence must not contain an invoice bundle")
        if dataset_commitment is not None:
            raw_commitment = dataset_commitment.removeprefix("sha256:") if isinstance(dataset_commitment, str) else dataset_commitment
            dataset_commitment = "sha256:" + _hash(raw_commitment, "dataset_commitment")
        contribution = {
            "participant_id": participant_id,
            "provider_id": provider_id,
            "run_id": self._run_id,
            "round_id": self._round_id,
            "operation": self._operation,
            "module_hash": self._module_hash,
            "workload_hash": workload_hash,
            "deal_id": _text(deal_id, "deal_id"),
            "artifact_count": artifact_count,
            "deal_state": "succeeded",
            "execution_state": "succeeded",
            "invoice_bundle_hash": invoice_hash,
            "dataset_commitment": dataset_commitment,
            **hashes,
        }
        encoded = _json_copy_bytes(contribution)
        previous = self._contributions.get(participant_id)
        if previous is not None:
            if previous != encoded:
                raise EvidenceError("conflicting contribution for an already staged participant")
            return False
        if self._output is not None:
            raise EvidenceError("published round evidence is immutable")
        # A single signed receipt cannot stand in for two independent participants.
        for item in self._contributions.values():
            if json.loads(item)["receipt_hash"] == hashes["receipt_hash"]:
                raise EvidenceError("receipt is already assigned to another participant")
        self._contributions[participant_id] = encoded
        return True

    def record_failure(self, participant_id: str, reason: str) -> None:
        """Retain an unsuccessful attempt; it does not count as a contribution."""
        self._participant(participant_id)
        reason = _text(reason, "failure reason")
        if self._output is not None:
            raise EvidenceError("published round evidence is immutable")
        if participant_id in self._contributions:
            raise EvidenceError("cannot replace a staged contribution with failure")
        history = self._failures.setdefault(participant_id, [])
        if reason not in history:
            history.append(reason)

    def snapshot(self) -> dict[str, Any]:
        """Return detached JSON data; an unpromoted snapshot never contains output."""
        missing = sorted(set(self._participants) - set(self._contributions))
        failed = [name for name in missing if self._failures.get(name)]
        return {
            "schema_version": "froglet.fl.reference_evidence.v1",
            "run_id": self._run_id,
            "round_id": self._round_id,
            "operation": self._operation,
            "module_hash": self._module_hash,
            "participants": dict(sorted(self._participants.items())),
            "expected_workload_hashes": dict(sorted(self._workload_hashes.items())),
            "status": "promoted" if self._output is not None else "ready" if not missing else "incomplete",
            "missing_participants": missing,
            "failed_participants": failed,
            "failure_history": {name: list(reasons) for name, reasons in sorted(self._failures.items())},
            "contributions": [json.loads(self._contributions[name]) for name in sorted(self._contributions)],
            "output": json.loads(self._output) if self._output is not None else None,
            "evidence_boundary": (
                "Application completeness record using caller-verified signed artifacts. "
                "Does not independently verify signatures, dataset identity, computation, "
                "output correctness, privacy or distributed rollback."
            ),
        }

    def promote(self, output: dict[str, Any]) -> dict[str, Any]:
        """Publish a detached output record only after every contribution is staged."""
        if type(output) is not dict:
            raise EvidenceError("output must be a JSON object")
        encoded = _json_copy_bytes(output)
        if self._output is not None:
            if encoded != self._output:
                raise EvidenceError("published output is immutable")
            return self.snapshot()
        missing = sorted(set(self._participants) - set(self._contributions))
        if missing:
            raise EvidenceError(f"cannot promote incomplete round; missing participants: {', '.join(missing)}")
        self._output = encoded
        return self.snapshot()
