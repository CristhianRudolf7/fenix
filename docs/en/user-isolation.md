# Accounts and isolated user runtimes

The shared gateway authenticates users and selects their account controller from a protected local registry. A client cannot select another tenant through a URL parameter, request header, tool call or model response. Each account has its own controller database, conversations, projects, skills, Telegram identity and schedules.

The current multi-user implementation separates controllers from task execution. Shell commands, terminals, language servers and Chromium run in constrained containers with selected mounts. See [execution isolation, installation and migration](execution-isolation.md) for the supported architecture, resource limits, browser policies, credential broker, encrypted checkpoints and measured limitations. Updating the repository does not migrate already running accounts.

## Account administration

There is a login screen, but **no registration screen or account-creation HTTP endpoint**. Provision accounts locally:

```sh
# The password is read from stdin, never from shell arguments.
fenix accounts create REGISTRY STAGING_USERS_ROOT EMAIL http://127.0.0.1:4180
fenix accounts disable REGISTRY EMAIL
fenix accounts enable REGISTRY EMAIL
fenix accounts passwd REGISTRY EMAIL
```

Run these commands as the registry's established trusted owner. Account creation initially prepares a directory; it does not create a quota-enforced volume or start a controller. For hardened deployments, the host operator moves that prepared directory into the configured account tree and provisions its fixed-capacity filesystem using the migration guide. Do not grant the gateway write access to the protected account-storage parent merely to create directories.

Use a unique loopback upstream port, UUID and upstream token for every account. The registry rejects duplicate mappings and reloads current state during authentication and proxying. Administrative writes take a lock and replace the registry atomically. Disablement, enablement and password changes advance the authorization revision. Reprovision a stopped controller with the current revision before starting it again.

Account directories include `projects`, `workspace`, `memory`, `skills`, `artifacts`, `sessions` and `state`. The actual conversation, memory and task database remains `state/fenix.sqlite3`; the named directories do not imply additional copies of that database. Skills are account-specific. Preserve UUIDs and take consistent database backups during migration. Stop the previous runtime and Telegram poller before enabling a replacement.

## Sessions and live revocation

Passwords use salted Argon2id hashes. The host registry contains a separate random upstream credential for each controller and stays outside tenant mounts. Never commit it. Cookies are HttpOnly and SameSite=Strict, with Secure on HTTPS deployments. Configure the gateway's established public origin, not a task runner origin.

Sessions expire exactly 168 hours (604,800 seconds) after successful login. Activity does not extend that deadline. The gateway persists sessions in `accounts.sessions.json` beside the registry with owner-only permissions and atomic writes. Browser and gateway restarts retain the original expiry. Logout durably revokes that session.

Sessions bind the stable account UUID, password hash and authorization revision. Reordering the registry cannot change their owner. Removing/disabling an account, changing its password or advancing its revision invalidates prior sessions; re-enabling cannot resurrect them. Active proxy streams recheck authorization, including a one-second check during quiet SSE periods. Keep the registry and session files outside account volumes and include them in trusted host backups.

Login attempts are throttled and password hashing uses bounded blocking workers. Same-origin checks apply to login and logout. Each controller requires its own upstream token for every HTTP request and exposes its listener only through a host-loopback port. The gateway applies bounded request concurrency and body/header deadlines while supporting long-running response streams.

## Deployment boundary

Use the root-installed `fenix-execution.service` and system `fenix-controller@ACCOUNT_UUID.service` described in the [migration guide](execution-isolation.md#installation-and-migration). Controller startup checks identity, current registry credentials/revision, approved image, fixed-capacity volume, mounts, resource limits, firewall and execution-service health. Docker's independent restart policy stays disabled. The gateway does not need root or Docker access.

Controllers retain their own provider credentials and state. Task runners receive selected project or browser mounts and a minimal environment, never the controller database, master credentials or control sockets. The host execution service and operator are trusted, and the measured OCI backend shares the host kernel. Standalone local mode does not provide this multi-user boundary. Public conversation sharing, arbitrary preview ports and automatic cross-account grants are not enabled.
