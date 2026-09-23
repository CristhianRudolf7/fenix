---
name: network-diagnostics
description: Diagnose DNS, routing, TCP, TLS and proxy failures by separating network layers.
metadata:
  version: 1.0.0
  category: operations
  tags: network, diagnostics
  aliases: red, DNS, TLS, certificado, puerto, proxy
  required_tools: file_read,file_write,command_run
  requires: Authorized host/network and available diagnostic commands
  effects: mixed
---
# Network Diagnostics

## Scope

Diagnose DNS, routing, TCP, TLS and proxy failures by separating network layers.

## Inputs

Source/destination, expected protocol, error and relevant proxy/tunnel configuration. Prerequisite: Authorized host/network and available diagnostic commands.

## Workflow

1. Check the requested hostname and resolution from the failing environment. Compare address family and split DNS before assuming the service is down.
2. Test connection establishment separately from TLS and application response. Preserve SNI/Host and certificate validation while diagnosing.
3. Inspect listener binding, routing, firewall and proxy/tunnel configuration within scope. An open port on another process is not the expected service.
4. Make only the identified correction and rerun from the original caller; record relevant timestamps and endpoint details.

## Verification

Confirm DNS, secure transport and a real protocol response without disabling verification.

## Limits

Do not scan unrelated networks, expose ports globally or disable TLS checking as a permanent fix.

## Español

Aísla DNS, conexión, TLS y aplicación; verifica desde el cliente original sin desactivar controles para ocultar el fallo.
