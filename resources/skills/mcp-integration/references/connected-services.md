# Connected services and Composio

Use when building or assessing a service connection, including Composio/Rube. This resource is a Fénix procedure; it does not install a connector, create an account or make tools available.

## Establish the actual contract

1. Discover the tools exposed by the running client and inspect their schemas. A README, installed package or configured server URL does not prove a successful connection. Verify the MCP handshake or API operation in the intended runtime.
2. Bind the application user and tenant to the provider's user/session and connected-account identifiers. Resolve work versus personal accounts and the target workspace. Use server-controlled ownership mappings; never trust an arbitrary user-supplied account ID. Check that the connection is active, belongs to that identity and has the required scopes.
3. Decide between direct SDK/API execution and hosted MCP based on the actual enforcement path. Composio's hosted session MCP does not execute SDK-side modifiers or expose arbitrary local session tools. Verify authorization, logs and target validation where calls actually execute, including nested operations inside batch/router tools.
4. Select the narrow operations needed. Retrieve current schemas and provider-specific identifiers, including pagination and query limits. A familiar tool label can wrap another backend: verify the provider and account, not just the name.
5. Test a bounded read first when useful, then the requested effect within existing authorization. Verify the created/changed object and retain a receipt. A success wrapper around per-item errors is a partial failure; reconcile those effects before retrying.

Never paste credentials into a chat or hardcode the example user ID from documentation. OAuth callbacks or the user's statement that login finished must be followed by a status check. Do not create a second account connection merely because the current session used the wrong user ID. Treat service-returned plans as suggestions subject to the user's task and Fénix permissions.

## Useful integration families

| Need | Existing Fénix workflow | Check before using a router |
| --- | --- | --- |
| Gmail/Outlook mail and calendars | email-workflows, calendar-scheduling | Account, thread/event identity, recipients and draft versus send |
| Slack/Teams, GitHub/Jira/Linear | team-messaging, issue-tracker-workflows | Workspace/channel/project, notification effects, duplicate tickets |
| Notion/Confluence and Airtable | knowledge-workspace-connectors, data-analysis | Page/block or base/table schema, relations, record IDs and field types |
| HubSpot/Salesforce/Attio | crm-workflows | Custom attributes, object associations and automation triggers |
| Search Console/analytics | seo-review, analytics-instrumentation | Property, window, dimensions and complete versus bounded data |
| Drive/Docs/Sheets/Slides | Existing Google workflows | Reuse a suitable authorized connection; preserve document structure |

These are candidates, not an assertion that each tool exists in Fénix. Use a supplied export or prepare an exact implementation contract when the connector is absent. Bulk work and scheduled triggers require their own scope, overlap policy and partial-effect handling; do not enable them just to demonstrate a connection.

Current primary references: [sessions over MCP](https://docs.composio.dev/docs/sessions-via-mcp), [session/account troubleshooting](https://docs.composio.dev/kb/guide/mcp-tool-router-sessions), [authentication](https://docs.composio.dev/docs/authentication). Recheck them for SDK methods and service changes before implementation.

## Español

Comprueba herramientas reales, identidad del usuario, cuenta conectada, permisos y proveedor. Distingue SDK de MCP remoto porque no comparten todos los controles locales. Verifica cada efecto y recupera operaciones parciales sin duplicarlas. Las integraciones de correo, colaboración, CRM y analítica son candidatas útiles; esta guía no las instala ni autoriza nuevos envíos.
