# Personal remote access

The agent server authenticates one owner directly. The multi-account gateway and `fenix gateway`/`fenix accounts` commands have been removed in the development version. Messaging and event delivery need no account proxy.

Stop the agent and configure its owner with `fenix owner EMAIL --password-stdin [--data-dir PATH]`, providing the password through standard input. Start `fenix` with the same data directory. There is no public registration endpoint. Reconfiguring the owner invalidates previous sessions.

Sessions expire exactly seven days after login, without sliding renewal, and survive restarts. Logout revokes one session; a password change revokes all sessions. SSE closes on expiration or revocation while agent tasks keep running.

For remote access, set `FENIX_PUBLIC_ORIGIN` to the HTTPS origin and route Caddy or your tunnel to the personal server. Cookies use HttpOnly, SameSite=Strict and Secure for HTTPS configuration. The frontend can remain a separate process. These changes have not been deployed or validated against the production firewall.
