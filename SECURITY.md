# Security Policy

## Reporting A Vulnerability

Please report security vulnerabilities privately to Vatsal Chavda rather than opening a public issue.

Especially sensitive issues include:

- API-key or Telegram bot-token disclosure.
- Telegram authorization bypass.
- Unauthorized local-file access.
- Exposure of profiles, preferences, memories, sessions or saved locations.
- Unsafe tool execution.
- Sensitive data included in logs or generated artifacts.

## Known Security Limitations

- Interactive approval enforcement for some local tools is under development.
- Symbolic-link escape protection for allowed filesystem paths is planned.
- Telegram tool-execution logs may contain sensitive local context.
- Moving Gemini API-key transport from URL query parameters to request headers is planned hardening.

## Safe Defaults

Remote Telegram access to high-risk local tools must remain disabled by default unless the user deliberately enables it and understands the risk.
