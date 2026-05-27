---
name: release-doc
description: Generate a progressive evolution documentation file in docs/ named evolution-NN-descriptor.md following the architecture.md format, covering the latest batch of commits since the last documented release.
---

# Release Documentation Skill

Use this skill to generate a progressive evolution documentation file capturing the latest stage of the Towergate project. It follows the same format and level of detail as `docs/architecture.md`.

## When to use

- A new feature or significant change has been committed to `main`.
- You want to produce a standalone documentation file (named `docs/evolution-NN-descriptor.md`) to complement `docs/architecture.md` and show the project's progression at a glance.
- You want to record architecture decisions, code changes, and design rationale for the most recent release batch.

## Instructions

1. **Identify the last documented commit.**  
   Read `docs/architecture.md` and find the sentence mentioning the last covered commit (e.g. "as of commit `XXXXXXX`"). That is your baseline.

2. **Gather the release commits.**  
   Run `git --no-pager log --oneline <last-documented-commit>..HEAD` to list all commits that are part of this release.  
   Also run `git --no-pager log <last-documented-commit>..HEAD --format="%H %s%n%b"` to get full messages and bodies.

3. **Identify the evolution index.**  
   List existing files in `docs/` matching the pattern `evolution-*.md`. The next file should increment the highest existing index by one.  
   If no evolution files exist (only `architecture.md`), start at `01`.

4. **Identify the headline feature.**  
   From the commit messages, determine the single most significant feature or evolution introduced. Condense it into a short kebab-case descriptor.

5. **Name the file.**  
   Use the format `docs/evolution-<NN>-<descriptor>.md` where `<NN>` is a zero-padded two-digit index and `<descriptor>` is the feature in kebab-case.  
   Example: if the release is the second evolution and features the terminal HUD, name the file `docs/evolution-02-terminal-hud-overlay.md`.  
   This keeps files sorted naturally and shows the project's progression at a glance.

6. **Analyze the diff.**  
   Run `git --no-pager diff <last-documented-commit>..HEAD -- src/` to see all source changes.  
   Also run `git --no-pager diff <last-documented-commit>..HEAD -- Cargo.toml` to see any dependency changes.

7. **Read all modified source files in full.**  
   Read every `.rs` file that was touched in the diff (both new and modified). Understand the code fully before writing the doc.

8. **Read `docs/architecture.md` for format reference.**  
   Follow the same structure, tone, and level of detail:
   - Use a top-level `# Title` with project name and brief subtitle.
   - Use tables for component overviews (feature flags, API changes, etc.).
   - Use code blocks for key type definitions, system signatures, etc.
   - Break down each module with `###` subsections.
   - Include a "Testing Strategy" section if tests were added/modified.
   - Include a "Git History — Development Narrative" table for this release's commits.
   - Include an "Architecture Decisions" section for any notable design choices.
   - End with any relevant build/run notes.

9. **Generate the file.**  
   Write the file to `docs/evolution-<NN>-<descriptor>.md` with the full documentation content. Ensure the file is comprehensive and mirrors the quality of `docs/architecture.md`.

10. **Validate.**  
   Run `cargo check` to make sure the code described still compiles (this is a sanity check on your understanding, not a test of the doc itself).