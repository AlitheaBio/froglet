# Hosted Trial (retired)

The first-party hosted trial at `try.froglet.dev`, with its `ai.froglet.dev`
origin, was retired on 8 October 2026. Its session creation had returned a
Cloudflare origin-DNS error since 24 September 2026, so it no longer produced
Froglet evidence. The public self-service beta at <https://froglet.dev/services/>
replaces it; see
[Public Beta Scope](../docs-site/src/content/docs/learn/cloud-trial.mdx).

The former public contract (shared session pool, five `demo.*` services, agent
prompt and failure taxonomy) is preserved in git history:
[HOSTED_TRIAL.md at f8d7e5e](https://github.com/AlitheaBio/froglet/blob/f8d7e5e/docs/HOSTED_TRIAL.md).

## What remains

- The node's hosted-trial routes and `FROGLET_HOSTED_TRIAL_*` settings are
  unchanged for operators who run their own trial; see
  [CONFIGURATION.md](CONFIGURATION.md). [openapi.yaml](openapi.yaml) still
  describes those routes.
- DNS for `try.froglet.dev` and `ai.froglet.dev`, and the separately deployed
  `try.froglet.dev` site, are outside this repository. Until they are removed or
  redirected, the live landing page and its `llms.txt` can still describe the
  retired flow.
