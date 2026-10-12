<!--
Thanks for the PR. Please fill in the sections below; delete the ones that
don't apply. If this is a draft, that's fine — mark it Draft so reviewers
know.

Security issues: do not open a PR that references an unpatched vulnerability.
Email security@alithea-bio.com first (see SECURITY.md).
-->

## Summary

<!--
1-3 sentences: what changed and why. Link the motivating issue or discussion,
or cite the internal tracking item in plain text if the source task lives
outside this public repo.
-->

## Related

<!--
Issues / Discussions this closes or advances. Use "closes #123" when a public
issue exists; otherwise cite the internal tracking item in plain text.
-->

## How this was verified

Tick what you ran locally. Unchecked items should be called out so a reviewer
knows whether to run them or whether they don't apply.

- [ ] `./scripts/release_gate.sh` — required software checks pass; skipped/external gates are listed
- [ ] `./scripts/release_gate.sh --compose` — compose-backed smoke passes (if this PR touches the provider/runtime/MCP surface)
- [ ] `./scripts/gitleaks_gate.sh` — tracked/non-ignored public source, candidate HEAD and selected public history are clean
- [ ] `cargo fmt --all --check` + `cargo clippy --all-targets -- -D warnings`
- [ ] Docs site still builds (`npm --prefix docs-site run build`) — if this PR touches `docs/` or `docs-site/`
- [ ] Relevant new tests added — at least one that fails against the pre-change behavior

## Risk and rollback

<!--
What's the blast radius if this is wrong? Is there a config flag to disable?
What's the rollback (revert the commit, roll back a deploy, rotate a key)?

Delete this section if the change is a pure doc or a strictly-additive test.
-->

## Breaking changes

<!--
Does this change any of: kernel artifact shapes, MCP tool signatures, HTTP
route paths, env var names, config file schemas, on-disk identity/storage
formats? If yes, list them and link to a migration note. If no, write "none".
-->

none

## Docs updated

- [ ] README or docs/ updated where user-visible behavior changed
- [ ] CHANGELOG.md `Unreleased` section updated (if user-visible)
- [ ] Public issue/docs tracking updated (if this closes a tracked public task)
