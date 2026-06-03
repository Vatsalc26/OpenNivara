# Mintlify Deployment

Public site: [https://story-0890af7b.mintlify.app/](https://story-0890af7b.mintlify.app/)

Local setup:

- Node.js must be installed. This repo has been checked with Node `v24.14.0`.
- The authenticated local CLI is `mintlify` (`mintlify@4.2.588` during the stabilization pass).
- `docs-site/package.json` exposes `bun run dev` as `mintlify dev`.
- `bun run docs:site:check` runs the repo's structural docs-site check without requiring cloud credentials.
- `mintlify validate` runs the Mintlify strict build validation when the CLI is authenticated.

Deployment:

- Mintlify CLI status reports organization `story` and subdomain `story-0890af7b`.
- Configure the Mintlify dashboard repo integration to deploy the main branch from `/docs-site`.
- Do not commit Mintlify tokens or secrets.
- CI runs the docs-site structural check in the quality workflow and the docs workflow.
