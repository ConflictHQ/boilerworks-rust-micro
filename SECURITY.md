# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Boilerworks, please report it responsibly.

**Do not open a public issue.**

Instead, email **security@weareconflict.com** with:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge your report within 48 hours and aim to release a fix within 7 days for critical issues.

## Supported Versions

| Version | Supported |
| ------- | --------- |
| latest  | Yes       |

## Security Best Practices

When deploying Boilerworks:

- Set `API_KEY_SEED` to a strong, unique value (never keep the `.env.example` default) and rotate the seed admin key after first boot
- Change the default Postgres credentials in `DATABASE_URL`
- Use HTTPS in production (terminate TLS at a reverse proxy in front of the service)
- Treat plaintext API keys as secrets — they are shown once at creation and only the SHA256 hash is stored
- Grant keys the narrowest scopes needed (`events.read`, `events.write`, `keys.manage`) instead of the `*` wildcard
