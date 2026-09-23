# Independent frontend and backend deployment

Production runs the frontend and API as separate **systemd user services**:

- `fenix-web.service`: Caddy on loopback port 4173, serving built React/Vite files
  from `~/.local/share/fenix-web/current` and proxying `/api` to port 4174.
- `fenix.service`: the Rust account gateway on loopback port 4174. It retains
  login, session validation, per-account routing and streaming.
- The existing system-level controller and execution services keep running the
  isolated agents. `fenix-tunnel.service` still forwards the public domain to 4173.

Caddy preserves Host, Origin, cookies and API paths. Set the gateway's existing
`FENIX_PUBLIC_ORIGIN` to the public HTTPS origin; do not remove origin validation.
These production templates assume HTTPS terminates at the existing tunnel.
Caddy's administrative endpoint is disabled and neither local port is public.

## Publish frontend changes

```sh
pnpm --dir apps/web install --frozen-lockfile
pnpm --dir apps/web build
bash scripts/publish-frontend.sh
```

The script copies a release and atomically switches `current`. It retains previous
asset names for already-open tabs. There is **no Rust build and no backend restart**.
Old release directories are kept for rollback; their disk usage should be reviewed
periodically. To roll back, atomically replace `current` with a symlink to a prior
release. HTML is revalidated, missing assets return 404, and `/api` never falls
back to HTML. Responses in streaming are forwarded without proxy buffering.

Templates are `packaging/linux/Caddyfile`, `packaging/linux/fenix-web.service`
and `packaging/linux/fenix-gateway.service` (installed as `fenix.service`).
Install them under `~/.config/fenix/` and `~/.config/systemd/user/` respectively.
Set `FENIX_GATEWAY_PORT=4174` and `FENIX_WEB_DIST=` in the gateway environment,
then run `systemctl --user daemon-reload` and enable/start `fenix-web.service`.
The initial migration requires coordinating the gateway port change; future
frontend releases do not. Keep the previous binary/configuration for rollback.

## Backend and local development

Rust no longer embeds or requires a built frontend at compile time. For local
single-process compatibility it can serve files from `FENIX_WEB_DIST`; an empty
value disables that fallback. Without the variable it reads `apps/web/dist` in
its source checkout. `scripts/install-local.sh` copies web files beside the local
installation instead of embedding them. Vite's development server remains an
option; it is not used in production.

Backend updates still require a Rust build and a coordinated restart of the
appropriate gateway/controller services. Frontend and API contracts must remain
compatible when released independently.

## Validation

`python3 scripts/check-frontend-proxy.py` checks SPA routing, API isolation,
missing assets, PDF MIME types, security headers, SSE delivery and changing a
frontend file without restarting Caddy. It requires Caddy and uses a local mock
backend, no model provider. Rust asset tests cover runtime replacement and path
traversal/symlink rejection. Production login and authenticated APIs are checked
separately after deployment.

Reference: [Caddy SPA and API patterns](https://caddyserver.com/docs/caddyfile/patterns).
