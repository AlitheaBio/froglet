#!/usr/bin/env python3
"""Synthetic immunopeptidomics counts over two local Froglet providers.

NVIDIA FLARE Executor/DXO components run in this Python process; Froglet nodes
execute the signed Wasm requests against separate provider-local SQLite files.
This is a federated analytics reference, not a provisioned FLARE deployment,
biological validation, differential-privacy guarantee, or proof of data identity.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from contextlib import closing
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO / "python" / "froglet-verify"))
sys.path.insert(0, str(REPO))
import froglet_verify
from python.froglet_fl_evidence import RoundEvidence
from examples import a2a_compute_demo as runtime

SCHEMA = "froglet.immunopeptidomics.counts.v1"
OPERATION = "immunopeptidomics_candidate_detection"
CAPABILITY = "db.sqlite.query.read.cohort"
FIXTURE = REPO / "examples/fixtures/immunopeptidomics/fixture.json"
COUNT_FIELDS = ("cohort_total", "hla_unknown", "hla_nonmatching", "eligible", "measured",
                "detected", "not_detected", "not_measured", "failed", "missing")
INTERPRETATION = {
    "hla_selection": "recorded_genotype_not_peptide_restriction",
    "not_detected": "not_detected_in_measured_assay_not_biological_absence",
    "denominator": "measured_eligible_samples",
}
TABLES = {
    "samples": ("sample_id", "sample_class", "organ", "assay_status"),
    "sample_hla_typing": ("sample_id", "allele_name"),
    "sample_hla_typing_status": ("sample_id", "locus", "status"),
    "sample_peptide_detections": ("sample_id", "sequence"),
}


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def validate_query(query):
    require(type(query) is dict and set(query) == {
        "candidate_peptide", "hla_allele", "sample_class", "organ"}, "Invalid query fields")
    require(all(type(value) is str and value for value in query.values()),
            "Query fields must be nonempty strings")
    require(re.fullmatch(r"[ACDEFGHIKLMNPQRSTVWY]{1,128}", query["candidate_peptide"]),
            "Expected an unmodified uppercase peptide sequence")
    require(all(query[key].strip() and len(query[key].encode("utf-8")) <= 128
                for key in ("sample_class", "organ")), "Cohort labels must contain 1..128 UTF-8 bytes")
    require(re.fullmatch(r"HLA-[ABC]\*[0-9]{2,3}:[0-9]{2,3}", query["hla_allele"]),
            "Expected an exact two-field HLA class-I allele")


def oracle(site, query):
    """Independent set-based calculation from fixture rows, without SQL or Wasm."""
    validate_query(query)
    require(type(site) is dict and set(site) == set(TABLES), "Invalid fixture tables")
    for table, columns in TABLES.items():
        require(type(site[table]) is list and all(
            type(row) is dict and set(row) == set(columns)
            and all(type(value) is str and value for value in row.values())
            for row in site[table]), f"Invalid {table} fixture rows")
    sample_ids = [row["sample_id"] for row in site["samples"]]
    require(len(sample_ids) == len(set(sample_ids)), "Duplicate fixture sample ID")
    require(all(row["sample_id"] in sample_ids for table in TABLES if table != "samples"
                for row in site[table]), "Orphan fixture observation")
    statuses = {}
    for row in site["sample_hla_typing_status"]:
        key = (row["sample_id"], row["locus"])
        require(key not in statuses, "Duplicate HLA typing status")
        require(row["status"] in {"complete", "incomplete", "missing"}, "Invalid typing status")
        statuses[key] = row["status"]
    alleles = {(row["sample_id"], row["allele_name"]) for row in site["sample_hla_typing"]}
    detections = {(row["sample_id"], row["sequence"]) for row in site["sample_peptide_detections"]}
    counts = dict.fromkeys(COUNT_FIELDS, 0)
    locus = query["hla_allele"].split("*", 1)[0]
    for row in site["samples"]:
        require(row["assay_status"] in {"measured", "not_measured", "failed"}, "Invalid assay status")
        if (row["sample_class"], row["organ"]) != (query["sample_class"], query["organ"]):
            continue
        counts["cohort_total"] += 1
        sample_id = row["sample_id"]
        if (sample_id, query["hla_allele"]) not in alleles:
            counts["hla_nonmatching" if statuses.get((sample_id, locus)) == "complete"
                   else "hla_unknown"] += 1
            continue
        counts["eligible"] += 1
        if row["assay_status"] == "measured":
            counts["measured"] += 1
            counts["detected" if (sample_id, query["candidate_peptide"]) in detections
                   else "not_detected"] += 1
        else:
            counts[row["assay_status"]] += 1
            counts["missing"] += 1
    return {"schema_version": SCHEMA, "query": dict(query), "counts": counts,
            "interpretation": dict(INTERPRETATION)}


def validate_result(result, query):
    require(type(result) is dict and set(result) == {
        "schema_version", "query", "counts", "interpretation"}, "Invalid aggregate result fields")
    require(result["schema_version"] == SCHEMA and result["query"] == query
            and result["interpretation"] == INTERPRETATION, "Result semantics differ from query")
    counts = result["counts"]
    require(type(counts) is dict and set(counts) == set(COUNT_FIELDS)
            and all(type(value) is int and value >= 0 for value in counts.values()),
            "Counts must be nonnegative integers")
    require(counts["cohort_total"] == counts["eligible"] + counts["hla_unknown"] + counts["hla_nonmatching"]
            and counts["eligible"] == counts["measured"] + counts["missing"]
            and counts["measured"] == counts["detected"] + counts["not_detected"]
            and counts["missing"] == counts["not_measured"] + counts["failed"],
            "Count denominators are inconsistent")


def aggregate_results(results, query):
    require(bool(results), "No site results")
    counts = dict.fromkeys(COUNT_FIELDS, 0)
    for result in results:
        validate_result(result, query)
        for key in COUNT_FIELDS:
            counts[key] += result["counts"][key]
    return {"schema_version": SCHEMA, "query": dict(query), "counts": counts,
            "interpretation": dict(INTERPRETATION)}


def detection_fraction(result):
    counts = result["counts"]
    return {"numerator": counts["detected"], "denominator": counts["measured"],
            "value": counts["detected"] / counts["measured"] if counts["measured"] else None}


def load_fixture(path):
    fixture = json.loads(Path(path).read_text(encoding="utf-8"))
    require(type(fixture) is dict and type(fixture.get("sites")) is dict
            and len(fixture["sites"]) == 2, "Reference fixture must contain exactly two sites")
    require(all(re.fullmatch(r"[a-z][a-z0-9_]{0,31}", name) for name in fixture["sites"]),
            "Invalid fixture site name")
    validate_query(fixture["query"])
    for site in fixture["sites"].values():
        oracle(site, fixture["query"])
    return fixture


def provision_sqlite(node, site):
    path = node.data / "cohort.sqlite"
    with closing(sqlite3.connect(path)) as connection:
        connection.executescript("""
            PRAGMA foreign_keys=ON;
            CREATE TABLE samples (sample_id TEXT PRIMARY KEY, sample_class TEXT NOT NULL,
                organ TEXT NOT NULL, assay_status TEXT NOT NULL
                CHECK (assay_status IN ('measured','not_measured','failed')));
            CREATE TABLE sample_hla_typing (sample_id TEXT REFERENCES samples(sample_id), allele_name TEXT NOT NULL);
            CREATE TABLE sample_hla_typing_status (sample_id TEXT REFERENCES samples(sample_id),
                locus TEXT NOT NULL, status TEXT NOT NULL CHECK(status IN ('complete','incomplete','missing')),
                PRIMARY KEY(sample_id,locus));
            CREATE TABLE sample_peptide_detections (sample_id TEXT REFERENCES samples(sample_id), sequence TEXT NOT NULL);
        """)
        for table, columns in TABLES.items():
            connection.executemany(f"INSERT INTO {table} VALUES ({','.join('?' for _ in columns)})",
                                   [tuple(row[column] for column in columns) for row in site[table]])
        connection.commit()
    path.chmod(0o600)
    policy = node.data / "wasm-policy.toml"
    policy.write_text("[sqlite]\nmax_queries_per_execution = 1\nmax_rows_per_query = 1000\n"
                      "max_result_bytes = 262144\n\n[sqlite.handles.cohort]\npath = "
                      + json.dumps(str(path.resolve())) + "\n", encoding="utf-8")
    policy.chmod(0o600)
    node.env["FROGLET_WASM_POLICY_PATH"] = str(policy)
    return path


def build_submission(module, query):
    validate_query(query)
    return {"schema_version": "froglet/v1", "submission_type": "wasm_submission",
            "workload": {"schema_version": "froglet/v1", "workload_kind": "compute.wasm.v1",
                         "abi_version": "froglet.wasm.host_json.v1", "module_format": "application/wasm",
                         "module_hash": hashlib.sha256(module).hexdigest(),
                         "input_format": "application/json+jcs", "input_hash": froglet_verify.payload_hash(query),
                         "requested_capabilities": [CAPABILITY]},
            "module_bytes_hex": module.hex(), "input": dict(query)}


def http_json(method, url, token, payload=None):
    data = json.dumps(payload).encode() if payload is not None else None
    request = urllib.request.Request(url, data=data, method=method,
        headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"})
    try:
        with runtime.HTTP.open(request, timeout=10) as response:
            content = response.read(4 * 1024 * 1024 + 1)
    except urllib.error.HTTPError as error:
        raise RuntimeError(f"Runtime request failed with HTTP {error.code}") from error
    require(len(content) <= 4 * 1024 * 1024, "Runtime response exceeds reference limit")
    return json.loads(content)


def submit(coordinator, provider_id, origin, submission, idempotency_key):
    token = (coordinator.data / "runtime/auth.token").read_text().strip()
    response = http_json("POST", coordinator.url + "/v1/runtime/deals", token, {
        "provider": {"provider_id": provider_id, "provider_url": origin},
        "offer_id": "execute.compute.host", "kind": "wasm", "max_price_sats": 0,
        "idempotency_key": idempotency_key, "submission": submission})
    deal = response["deal"]
    runtime_ms = deal["quote"]["payload"]["execution_limits"]["max_runtime_ms"]
    deadline = time.monotonic() + max(30, runtime_ms / 1000 + 15)
    while deal["status"] not in {"succeeded", "failed", "rejected", "canceled"}:
        require(time.monotonic() < deadline, "Runtime receipt polling timed out")
        time.sleep(0.05)
        deal = http_json("GET", coordinator.url + "/v1/runtime/deals/"
                         + urllib.parse.quote(deal["deal_id"], safe=""), token)["deal"]
    require(deal["status"] == "succeeded", "Signed computation did not succeed: "
            + str(deal.get("receipt", {}).get("payload", {}).get("failure_code", deal["status"])))
    return deal


def artifact_bundle(origin, deal):
    artifacts = {name: deal[name] for name in ("quote", "deal", "receipt")}
    for name in ("descriptor", "offer"):
        wrapped = runtime.get_json(origin + "/v1/artifacts/" + deal["quote"]["payload"][name + "_hash"])
        document = wrapped["document"]
        artifacts[name] = json.loads(document) if isinstance(document, str) else document
    return {"deal_id": deal["deal_id"], "result_hash": deal["result_hash"], "signed_artifacts": artifacts}


def verify_artifact_chain_offline(evidence, result):
    """Verify signatures, chain links, successful execution and exact JCS result."""
    artifacts = evidence["signed_artifacts"]
    require(set(artifacts) == {"descriptor", "offer", "quote", "deal", "receipt"},
            "Expected a free five-artifact chain")
    chain = froglet_verify.validate_full_chain(**artifacts)
    require(chain.get("valid") is True, "Offline signed-artifact verification failed")
    receipt, quote, deal = artifacts["receipt"], artifacts["quote"], artifacts["deal"]
    result_hash = froglet_verify.payload_hash(result)
    require(result_hash == evidence["result_hash"] == receipt["payload"].get("result_hash"),
            "Result differs from signed Receipt hash")
    require(evidence["deal_id"] == deal["hash"], "Deal identifier differs from signed Deal")
    require(receipt["payload"]["deal_state"] == "succeeded"
            and receipt["payload"]["execution_state"] == "succeeded", "Receipt is not successful")
    return {"offline_chain_verified": True, "artifact_count": 5,
            **{name + "_hash": artifact["hash"] for name, artifact in artifacts.items()},
            "invoice_bundle_hash": None, "result_hash": result_hash,
            "provider_id": receipt["signer"], "requester_id": deal["signer"],
            "workload_hash": quote["payload"]["workload_hash"],
            "module_hash": receipt["payload"]["executor"].get("module_hash"),
            "deal_state": receipt["payload"]["deal_state"],
            "execution_state": receipt["payload"]["execution_state"]}


def nvflare_components():
    """Fail explicitly: this NVIDIA reference has no substitute execution mode."""
    try:
        from nvflare.apis.dxo import DXO, DataKind, from_shareable
        from nvflare.apis.executor import Executor
        from nvflare.apis.fl_context import FLContextManager
        from nvflare.apis.signal import Signal
        version = importlib.metadata.version("nvflare")
    except (ImportError, importlib.metadata.PackageNotFoundError) as error:
        raise RuntimeError("NVIDIA FLARE is required; install python/requirements-immunopeptidomics.txt") from error
    require(version == "2.9.0", "This reference requires the tested NVIDIA FLARE 2.9.0")
    return DXO, DataKind, from_shareable, Executor, FLContextManager, Signal


def dispatch_nvflare(site_names, query, worker):
    """Call installed FLARE APIs; worker performs real Froglet transport in run()."""
    DXO, DataKind, from_shareable, Executor, FLContextManager, Signal = nvflare_components()

    class SiteExecutor(Executor):
        def __init__(self, name):
            super().__init__()
            self.site_name = name

        def execute(self, task_name, shareable, fl_ctx, abort_signal):
            require(task_name == OPERATION and not abort_signal.triggered, "Invalid or aborted FLARE task")
            request = from_shareable(shareable)
            require(request.data_kind == DataKind.APP_DEFINED and request.data == query,
                    "FLARE task differs from declared query")
            return DXO(DataKind.APP_DEFINED, data=worker(self.site_name, request.data)).to_shareable()

    def execute(name):
        manager = FLContextManager(engine=None, identity_name=name, job_id="froglet-immunopeptidomics")
        with manager.new_context() as context:
            response = SiteExecutor(name).execute(OPERATION, DXO(DataKind.APP_DEFINED, data=dict(query)).to_shareable(),
                                                  context, Signal())
        result = from_shareable(response)
        require(result.data_kind == DataKind.APP_DEFINED, "Invalid FLARE response kind")
        return result.data

    records, failures = {}, {}
    with ThreadPoolExecutor(max_workers=len(site_names)) as pool:
        pending = {pool.submit(execute, name): name for name in site_names}
        for future in as_completed(pending):
            name = pending[future]
            try:
                records[name] = future.result()
            except Exception as error:
                failures[name] = {"type": type(error).__name__, "message": str(error)}
    return records, failures


def stage_records(evidence, records, failures, *, run_id, module_hash, workload_hashes, fixture_sites):
    for name, failure in failures.items():
        evidence.record_failure(name, failure["type"] + ": " + failure["message"])
    for name, record in records.items():
        audit = verify_artifact_chain_offline(record, record["result"])
        arguments = {"run_id": run_id, "round_id": 0, "operation": OPERATION,
                     "module_hash": module_hash, "workload_hash": workload_hashes[name],
                     "deal_id": record["deal_id"], "verification": audit,
                     "dataset_commitment": froglet_verify.payload_hash(fixture_sites[name])}
        require(evidence.stage(name, **arguments), "Contribution unexpectedly already staged")
        require(evidence.stage(name, **arguments) is False, "Identical staged retry was not idempotent")
        record["offline_verification"] = audit


def write_report(path, report):
    if path is None:
        return
    path = Path(path)
    encoded = json.dumps(report, indent=2, allow_nan=False) + "\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix="." + path.name + ".", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            output.write(encoded)
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def source_hashes():
    paths = [Path(__file__), REPO / "examples/a2a_compute_demo.py", REPO / "examples/research_profile.py",
             REPO / "python/froglet_fl_evidence.py", REPO / "python/requirements-immunopeptidomics.txt",
             REPO / "examples/wasm-services/Cargo.toml", REPO / "examples/wasm-services/Cargo.lock"]
    paths += sorted((REPO / "python/froglet-verify/froglet_verify").glob("*.py"))
    paths += sorted((REPO / "examples/wasm-services/immunopeptidomics").rglob("*.rs"))
    paths += [REPO / "examples/wasm-services/immunopeptidomics/Cargo.toml"]
    return {str(path.relative_to(REPO)): sha256(path) for path in paths}


def run(binary, module_path, fixture_path=FIXTURE, output=None, keep_state=False,
        execution_timeout_secs=30, unavailable_site=None):
    require(type(execution_timeout_secs) is int and 1 <= execution_timeout_secs <= 300,
            "Execution timeout must be an integer from 1 through 300")
    nvflare_components()
    fixture = load_fixture(fixture_path)
    names, query = sorted(fixture["sites"]), fixture["query"]
    require(unavailable_site is None or unavailable_site in names, "Unknown unavailable site")
    module = Path(module_path).read_bytes()
    require(module.startswith(b"\0asm\x01\0\0\0") and len(module) <= 1024 * 1024,
            "Expected a Wasm v1 module no larger than 1 MiB")
    module_hash = hashlib.sha256(module).hexdigest()
    submission = build_submission(module, query)
    workload_hashes = {name: froglet_verify.payload_hash(submission["workload"]) for name in names}
    run_id = "immunopeptidomics-" + secrets.token_hex(12)
    root = Path(tempfile.mkdtemp(prefix="froglet-immunopeptidomics-")).resolve()
    root.chmod(0o700)
    coordinator = runtime.Node(Path(binary), root, "coordinator", "runtime")
    nodes = {name: runtime.Node(Path(binary), root, name, "provider") for name in names}
    for node in nodes.values():
        node.env.update({"FROGLET_EXECUTION_TIMEOUT_SECS": str(execution_timeout_secs),
                         "FROGLET_PROVIDER_MAX_TOTAL_RUNTIME_MS": str(10 * execution_timeout_secs * 1000)})
    succeeded, evidence = False, None
    report = {"schema_version": "froglet.immunopeptidomics.reference_run.v1", "run_id": run_id,
              "status": "running", "query": query, "execution_timeout_secs": execution_timeout_secs,
              "source_sha256": source_hashes(), "fixture_sha256": sha256(fixture_path),
              "binary_sha256": sha256(binary), "module_sha256": module_hash,
              "framework": {"name": "NVIDIA FLARE", "version": importlib.metadata.version("nvflare"),
                            "components": ["Executor", "DXO", "Shareable", "FLContext"],
                            "scope": "in-process FLARE components; live localhost Froglet providers"},
              "limitations": ["All fixtures are synthetic; no biological validation.",
                              "Recorded sample HLA genotype does not establish peptide restriction.",
                              "Denominators count synthetic specimens, not unique donors or technical replicates.",
                              "Non-detection is not biological absence; missing assays are excluded from the measured denominator.",
                              "Small aggregate counts are disclosed; no differential privacy or secure aggregation.",
                              "Hashes bind local files and signed reported values, not physical dataset identity.",
                              "Completeness state is in memory; no durable resume or distributed rollback."],
              "workers": {}, "aggregate": None}
    phase = "setup"
    try:
        requester_id = coordinator.identity()
        identities = {name: node.identity() for name, node in nodes.items()}
        require(len({requester_id, *identities.values()}) == len(names) + 1, "Node identities are not independent")
        evidence = RoundEvidence(run_id=run_id, round_id=0, operation=OPERATION,
                                 module_hash=module_hash, participants=identities, workload_hashes=workload_hashes)
        provider_config, origins = [], {}
        report["provider_fixture_hashes"] = {}
        for name, node in nodes.items():
            database = provision_sqlite(node, fixture["sites"][name])
            report["provider_fixture_hashes"][name] = {"rows_jcs_sha256": froglet_verify.payload_hash(fixture["sites"][name]),
                                                       "sqlite_sha256": sha256(database)}
            node.start()
            origin = origins[name] = node.url
            offer = next(item for item in runtime.get_json(origin + "/v1/provider/offers")["offers"]
                         if item["payload"]["offer_id"] == "execute.compute.host"
                         and item["payload"]["settlement_method"] == "none")
            require(offer["payload"]["price_schedule"] == {"base_fee_msat": 0, "success_fee_msat": 0},
                    "Reference requires a free local compute offer")
            node.stop()
            node.env["FROGLET_LISTEN_ADDR"] = origin.removeprefix("http://")
            token = secrets.token_urlsafe(36)
            config = node.data / "a2a.json"
            runtime.private_json(config, {"clients": [{"requester_id": requester_id, "token": token,
                                                       "offer_hashes": [offer["hash"]]}]})
            node.env["FROGLET_A2A_CONFIG_PATH"] = str(config)
            provider_config.append({"provider_url": origin, "token": token, "allow_loopback": True})
        config = coordinator.data / "a2a.json"
        runtime.private_json(config, {"providers": provider_config})
        coordinator.env["FROGLET_A2A_CONFIG_PATH"] = str(config)
        for name, node in nodes.items():
            node.start()
            require(node.url == origins[name], "Provider origin changed on restart")
        coordinator.start()
        if unavailable_site:
            nodes[unavailable_site].stop()
        phase = "collect_site_counts"

        def worker(name, request):
            require(request == query, "Worker query changed")
            key = run_id + ":" + name
            deal = submit(coordinator, identities[name], origins[name], submission, key)
            result = deal["result"]
            validate_result(result, query)
            record = dict(artifact_bundle(origins[name], deal), result=result)
            verify_artifact_chain_offline(record, result)
            require(result == oracle(fixture["sites"][name], query), "Site result differs from independent oracle")
            replay = submit(coordinator, identities[name], origins[name], submission, key)
            require(all(replay[field] == deal[field] for field in ("deal_id", "deal", "quote", "receipt", "result_hash", "result")),
                    "Idempotent submission changed signed result")
            record["idempotent_replay_same_deal"] = True
            return record

        records, failures = dispatch_nvflare(names, query, worker)
        report["workers"], report["worker_failures"] = records, failures
        stage_records(evidence, records, failures, run_id=run_id, module_hash=module_hash,
                      workload_hashes=workload_hashes, fixture_sites=fixture["sites"])
        require(evidence.snapshot()["status"] == "ready", "Missing or failed site: aggregate not promoted")
        phase = "verify_and_promote"
        reduced = aggregate_results([records[name]["result"] for name in names], query)
        pooled = {table: [row for name in names for row in fixture["sites"][name][table]] for table in TABLES}
        require(reduced == oracle(pooled, query), "Pooled independent oracle disagrees with reduction")
        evidence.promote(reduced)
        report.update(status="succeeded", aggregate=reduced, detection_fraction=detection_fraction(reduced),
                      evidence=evidence.snapshot(), independent_oracle_matches=True)
        write_report(output, report)
        succeeded = True
        return report
    except BaseException as error:
        report.update(status="failed", phase=phase, aggregate=None,
                      error={"type": type(error).__name__, "message": str(error)},
                      evidence=evidence.snapshot() if evidence else None, retained_state_directory=str(root))
        write_report(output, report)
        raise
    finally:
        coordinator.stop()
        for node in nodes.values():
            node.stop()
        if succeeded and not keep_state:
            shutil.rmtree(root)
        else:
            print(f"Retained demo state directory: {root}", file=sys.stderr)


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--module", type=Path)
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--keep-state", action="store_true")
    parser.add_argument("--execution-timeout-secs", type=int, default=30,
                        help="Provider budget including cold compilation, 1..300 seconds (default30)")
    parser.add_argument("--unavailable-site", help="Stop this synthetic provider before dispatch to test incomplete output")
    args = parser.parse_args(argv)
    if bool(args.binary) != bool(args.module):
        parser.error("--binary and --module must be supplied together")
    if not 1 <= args.execution_timeout_secs <= 300:
        parser.error("--execution-timeout-secs must be between 1 and 300")
    return args


def main(argv=None):
    args = parse_args(argv)
    nvflare_components()
    binary = args.binary.resolve() if args.binary else REPO / "target/debug/froglet-node"
    module = args.module.resolve() if args.module else REPO / "examples/wasm-services/target/wasm32-unknown-unknown/release/immunopeptidomics.wasm"
    if not args.no_build and not args.binary:
        commands = [
            ["cargo", "build", "--locked", "-p", "froglet", "--bin", "froglet-node", "--target-dir", str(REPO / "target")],
            ["cargo", "build", "--locked", "--release", "--target", "wasm32-unknown-unknown", "-p", "immunopeptidomics",
             "--manifest-path", str(REPO / "examples/wasm-services/Cargo.toml"),
             "--target-dir", str(REPO / "examples/wasm-services/target")]]
        for command in commands:
            subprocess.run(command, cwd=REPO, env=runtime.clean_environment(), check=True)
    require(binary.is_file() and module.is_file(), "Missing node binary or Wasm module")
    report = run(binary, module, args.fixture, args.output, args.keep_state,
                 args.execution_timeout_secs, args.unavailable_site)
    print(json.dumps(report, indent=2, allow_nan=False))
    return 0


if __name__ == "__main__":
    def interrupted(_signum, _frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupted)
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        print("Reference run interrupted; started nodes stopped.", file=sys.stderr)
        raise SystemExit(130)
    except (OSError, RuntimeError, ValueError, KeyError, StopIteration, subprocess.SubprocessError) as error:
        print(f"Reference run failed: {error}", file=sys.stderr)
        raise SystemExit(1)
