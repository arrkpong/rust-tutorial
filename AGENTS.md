# Agents Guide (OpenAI-Aligned)

This file captures how to work with automated agents (e.g., ChatGPT/Codex) in this repo while following OpenAI’s recommended practices for safety, privacy, and reliability.

## Scope
- Assist with code generation, reviews, docs, and troubleshooting.
- Never handle secrets, credentials, or production-only data.
- Keep agents out of release-signing, irreversible data actions, or policy decisions without human review.

## Safety & Privacy
- **No secrets in prompts**: redact API keys, passwords, tokens, or private user data before sending to the model.
- **Minimize data**: share only the code or logs needed for the task; avoid full database dumps or PII.
- **Output scrutiny**: treat model replies as suggestions—review for correctness, licensing, and security before use.
- **Security posture**: do not run shell commands that could be destructive (e.g., `rm -rf`, privileged ops) unless explicitly required and reviewed.

## Prompting Checklist
- Be specific: include file paths, error messages, expected behavior, and constraints.
- State tech context: Rust 2024, Actix-web, SeaORM, Postgres, Docker Compose.
- Clarify desired format: e.g., “return patch only”, “give curl example”, or “summarize issues”.
- Ask for tests or verification steps when appropriate.

## Data Handling
- Strip environment files and secrets (`.env`, tokens) from context.
- When sharing logs, remove IPs, emails, phone numbers, and user identifiers unless sanitized.
- Avoid uploading compiled artifacts (`target/`, binaries).

## Verification
- Prefer `cargo test` / targeted checks; for this project, at minimum run the migration crate as needed and start the server locally before changes ship.
- Validate security-sensitive code (auth, hashing, DB access) with peer review in addition to agent output.

## Style & Docs
- Keep new content ASCII-only unless required.
- Add concise comments only when the code is not self-explanatory.
- Reference files with inline code paths (e.g., `src/main.rs`) for clarity.

## Change Control
- Review patches locally; avoid committing generated code without inspection.
- Keep commits scoped and reversible; no force pushes to shared branches.
- If an agent suggests deleting user changes you didn’t make, stop and consult a human.

## When to Escalate to Humans
- Unclear requirements or conflicting instructions.
- Security-sensitive flows (auth, secrets management, database migrations on prod).
- Destructive actions, data migrations, or schema changes without backups.
