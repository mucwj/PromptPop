# Security Policy

PromptPop is local-first: prompts, settings, exports, and backups are stored on
the user's device.

## Supported Versions

PromptPop is pre-1.0. Security fixes target the current `main` branch until a
formal release policy exists.

## Reporting a Vulnerability

Please report suspected vulnerabilities privately before opening a public issue.
Until a project security contact is published, send the report to the repository
owner through GitHub.

Useful details include:

- PromptPop version or commit
- Operating system and architecture
- Reproduction steps
- Expected impact
- Whether local prompt data, clipboard contents, filesystem access, or
  automation permissions are involved

## Security Notes

- Prompt content may contain private workflows, client context, credentials, or
  internal process knowledge. Treat exported prompt files and diagnostics as
  sensitive.
- Auto paste uses OS automation permissions on macOS. Users should only grant
  Accessibility access if they understand the behavior.
- The project should not add telemetry, remote sync, or external network calls
  without explicit documentation and user control.
