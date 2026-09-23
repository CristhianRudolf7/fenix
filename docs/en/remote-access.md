# Remote access with Cloudflare Tunnel

Build the frontend before the Rust server, as described in the README. The compiled frontend and API are served by the same Rust process; `fenix.service` manages both. Vite is a development server and is not needed in production.

Run the shared authentication gateway with `fenix gateway ~/.config/fenix/accounts.json`, using `packaging/linux/fenix-gateway.service` as the application service template. Provision accounts locally as described in [User isolation](user-isolation.md). The gateway serves the email/password login and requires a session for private APIs. Do not point the public tunnel at a standalone runtime or a tenant port.

Set `FENIX_PUBLIC_ORIGIN=https://your-host.example.com` in the gateway service environment to accept that exact HTTPS host and origin. Local loopback access remains available. Other hosts, mismatched origins and cross-site requests remain rejected. This setting selects the permitted origin; authentication is provided by the gateway.

Configure a named Cloudflare Tunnel to the gateway at `http://127.0.0.1:4173`, retaining the public Host header. Keep the final tunnel ingress rule as `http_status:404` and point a proxied CNAME for the hostname to `<tunnel-id>.cfargotunnel.com`. Cloudflare Access is not required: visitors should reach Fenix's own login directly.

When migrating a hostname previously protected by Access, first verify that the gateway rejects unauthenticated private API requests. Back up the Access application, policies and tunnel configuration, remove `originRequest.access` from that hostname's ingress rule, and remove the Access application for that hostname. Check for inherited Access requirements if the tunnel uses global defaults. Preserve unrelated hostnames and policies. Leaving required JWT validation in the tunnel would reject requests after removing the Access application. An additional Access login is optional only when explicitly desired; the production Fenix hostname does not use it.

Store the tunnel token in `~/.config/fenix/cloudflared.token` with mode `0600`. Use `packaging/linux/fenix-tunnel.service` as a user-service template; adapt the cloudflared executable path if necessary. The application service may use `~/.config/fenix/server.env`. Use the existing data directory to preserve history, model settings and Telegram pairing; do not replace it with a new empty installation.

Enable both services with `systemctl --user enable --now fenix.service fenix-tunnel.service`. User lingering must be enabled for startup without an interactive login. Inspect state with `systemctl --user status fenix.service fenix-tunnel.service` and logs with `journalctl --user -u fenix -u fenix-tunnel`.

Validate in a fresh browser session that the public root returns the Fenix login without redirecting to Cloudflare Access, unauthenticated `/api/auth/session` and private API requests return `401`, email/password login succeeds, and logout makes private APIs inaccessible again. Check that frontend assets load and authenticated SSE endpoints stay connected. Browser microphone APIs need HTTPS and browser permission. The host computer and its network must remain available. Restarting Fenix interrupts active work, so apply binary updates when tasks are idle. Remotely managed tunnel configuration changes do not require restarting the agent.

[Español](../es/remote-access.md)
