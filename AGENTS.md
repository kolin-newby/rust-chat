# AGENTS.md

Project conventions for any AI coding agent working in this repo.

## Commit messages

Commit messages MUST follow the Conventional Commits format: `type(scope): description`.

- `type` is one of: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert
- `scope` is optional but recommended
- Use imperative mood in the description (e.g. "add", not "added")
- Add a body/footer for additional context or breaking changes when needed

See the `conventional-commit` skill for the full structure and examples.

Commits should include a `Co-Authored-By: <current model, e.g. Claude Sonnet 5>` trailer, without an email address after the name.

## Commit chunking

When asked to commit changes, split them into separate logical commits by default (e.g. bugfixes separate from features, tests separate from the code they cover, unrelated tooling/config separate from source changes) rather than one commit covering everything. Only combine changes into one commit when they're too interdependent to build/compile separately (e.g. a type change and every call site it touches).

## Pre-commit review

Before running any commit, review the actual diff/file contents (not just filenames) of everything about to be staged and warn about:

- Likely secrets or credentials: API keys, tokens, private keys, passwords, connection strings with embedded credentials, `.env` files, cloud provider credential files
- Unusual or risky file types for this repo: binaries, archives, large data files, IDE/OS cruft, anything that looks auto-generated or out of place

Flag anything found before committing and wait for confirmation on how to proceed - don't silently exclude or silently include it.

## Documentation upkeep

After committing changes, check whether they affect any of these, and update independently as needed:

- **Architecture diagrams** (`docs/architecture/` - Level 1 Context, Level 2 Container, Level 3 Component, C4 model, Mermaid + Markdown): nodes, the wiring between them, implemented-vs-planned status, or per-component test coverage. Use the `c4-model` skill's Update mode.
- **README** (`README.md`): features, CLI commands/flags, quick start steps, testing instructions, or anything else it documents. Use the `good-readme` skill's Improve mode.
- **AI involvement section** (bottom of `README.md`): update when a model/agent contributes to this repo for the first time (add its entry), when an existing model's contribution areas shift meaningfully, or when the installed skills list changes. Don't re-tally commit counts on every commit - that's noise, not upkeep. When you do update it, cite actual numbers from `git log` and `skills-lock.json` rather than estimating.

For whichever needs updating, bring it up to date, then stage and commit that update immediately in its own commit - each gets a separate commit from the others too, per the commit-chunking rule above. If nothing changed for any of them, just note "no documentation changes needed" and move on without asking.
