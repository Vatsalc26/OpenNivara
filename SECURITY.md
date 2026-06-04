# Security Policy

## Reporting A Vulnerability

Please report security vulnerabilities privately instead of opening a public issue.

Current verified contact: send a direct message on X to [@choco_chip2m](https://x.com/choco_chip2m).

Especially sensitive issues include:

- API-key or Telegram bot-token disclosure.
- Telegram authorization bypass.
- Unauthorized local-file access.
- Exposure of profiles, preferences, memories, sessions or saved locations.
- Unsafe tool execution.
- Sensitive data included in logs or generated artifacts.

Do not include secrets, raw credentials, `.env` files, private local databases, or full sensitive logs in an initial report. Start with a concise summary and a safe reproduction outline.

## Known Security Limitations

- Interactive approval enforcement for some local tools is under development.
- Symbolic-link escape protection for allowed filesystem paths is planned.
- Telegram tool-execution logs may contain sensitive local context.
- Moving Gemini API-key transport from URL query parameters to request headers is planned hardening.

## Safe Defaults

Remote Telegram access to high-risk local tools must remain disabled by default unless the user deliberately enables it and understands the risk.
