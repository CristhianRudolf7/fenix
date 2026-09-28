---
name: cloud-deployment
description: Deploy an application to the selected cloud while preserving account, region, identity and rollback
  scope.
metadata:
  version: 1.1.0
  category: operations
  tags: cloud, deployment
  aliases: desplegar nube, AWS, Azure, GCP, Cloud Run, Cloudflare, Google Cloud, deployment identity, identidad de servicio, proyecto GCP
  required_tools: file_read,file_write,command_run
  requires: Configured provider CLI/API and authorized account
  effects: mixed
---
# Cloud Deployment

## Scope

Deploy an application to the selected cloud while preserving account, region, identity and rollback scope.

## Inputs

Provider/project, region, artifact, service identity and desired exposure. Prerequisite: Configured provider CLI/API and authorized account.

## Workflow

1. Resolve the exact account/project and deployment target. Read existing configuration and current provider documentation instead of guessing flags.
2. Separate build identity from runtime identity and configure secrets through the platform mechanism. Choose public/private exposure from the user requirement.
3. Deploy a versioned artifact and inspect asynchronous operation status. Preserve the previous working revision and understand data compatibility.
4. Check health through the intended public/private route, including DNS/TLS and dependent services. Report the actual revision and URL.

For Google Cloud, load [project and identity selection](references/google-cloud.md). Check the installed CLI's exact subcommand help or the corresponding current official API documentation. Keep project, location, deployment identity and runtime identity explicit; a configured default or a documentation example is not a target selection.

## Verification

Verify one meaningful user operation and the deployed version; distinguish DNS propagation from application failure.

## Limits

Do not enable paid services, expand permissions or make a private endpoint public as an implicit workaround.
