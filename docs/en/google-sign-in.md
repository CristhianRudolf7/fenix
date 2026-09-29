# Google sign-in and account registration

Fénix supports Google authentication at the host gateway, separately from Google
Workspace authorization. Login requests only `openid email profile`; Drive,
Gmail and Calendar still require explicit consent in Connections.

## Operator setup

Create a Google Cloud web OAuth client. Configure the authorized JavaScript
origin as `https://fenix.crehlabs.com` and register these exact redirect URIs:

- `https://fenix.crehlabs.com/api/auth/google/callback` — account access.
- `https://fenix.crehlabs.com/api/connectors/oauth/callback` — optional Workspace access.

Set `FENIX_GOOGLE_CLIENT_ID` and `FENIX_GOOGLE_CLIENT_SECRET` in the gateway's
private systemd environment file, alongside `FENIX_PUBLIC_ORIGIN`. Restart the
gateway after configuring them. Never expose the secret in Vite variables or
commit it. The connections backend also needs these credentials separately;
configuring the gateway does not automatically configure tenant containers.
Without credentials the login screen retains password access and explains why
Google is not yet available. Local development uses
`http://127.0.0.1:PORT/api/auth/google/callback`.

## New accounts and isolated capacity

Registration claims a **fresh, pre-provisioned tenant**, atomically under the
same registry lock used by administration. It never shares another user's
workspace and the public gateway never receives root/Docker privileges.

Prepare unused capacity with the existing operator workflow:

1. Run `fenix accounts reserve REGISTRY STAGING_USERS_ROOT slot-N@signup.invalid http://127.0.0.1:PORT`.
   This creates a new UUID and inaccessible random password; it does not read a
   password from stdin. Never convert an existing user's tenant into a reserve.
2. As the host operator, move the empty staged directory into the protected
   account storage tree. Provision that new UUID's distinct UID/GID, reserved disk capacity and execution
   policy using the normal isolation deployment instructions. Use
   `tenant-storage.py ... migrate` for the empty directory created by the CLI,
   then `tenant-container.py` and its systemd controller service as documented in
   [execution isolation](execution-isolation.md). Reserve sufficient host disk/RAM first.
3. Start the isolated controller with its normal settings. Registration checks
   its authenticated status endpoint before it can claim the slot.

Without ready capacity the user sees a retryable message; no partial account is
created and no existing data is reassigned. Multiple registrations cannot claim
the same tenant. A claimed slot is never automatically recycled. This admission
pool deliberately reuses the existing isolation infrastructure instead of
introducing a privileged provisioning API. Automatic pool replenishment is not
part of this change.

## Identity and sessions

- Authorization Code flow, PKCE S256, ten-minute single-use state, and a
  browser-bound HttpOnly cookie. Public callbacks serve a static landing page;
  only its same-origin POST completes authentication.
- Identity comes from Google's UserInfo endpoint using a token obtained by the
  gateway itself; client-submitted profiles and decoded JWTs are never trusted.
- Stable Google `sub` identifies returning users. Verified Gmail or hosted
  Workspace identities can link to their existing email account. Other email
  providers require signing into the existing account and linking in Profile.
- Registration is explicit through **Create account**. Login alone does not
  silently create an account.
- Google-only accounts have no usable local password. Existing password access
  remains unchanged; administrator password provisioning still works.
- Successful login creates the same durable, non-sliding seven-day session.
  Account disablement, logout and password changes retain existing semantics.
- Pending requests are bounded; repeated browser clicks replace pending state;
  concurrent external exchanges are limited. Provider errors/tokens are not
  returned to the user or logged.

Validation uses mocked Google token/UserInfo responses and frontend fixtures.
A real Google account round trip remains pending operator credentials/consent.
