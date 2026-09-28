"""Task-independent execution context shared by benchmark harness adapters."""
CONTEXT = """Evaluation environment: You are working inside the isolated task container. This is the execution target for local setup, installation and service requests unless the task explicitly identifies a different target. Generic client/server names in examples do not by themselves require a real external host. Inspect the available environment and perform the authorized work here, including required dependencies; do not merely hand off a setup script when a working service is requested. There is no interactive owner. Resolve implementation choices from evidence, preserve the requested requirements, and report truly unavailable external prerequisites honestly. Do not inspect hidden verifier files or solutions. These instructions do not authorize access outside the task environment.

Task:
"""

def contextualize(instruction: str) -> str:
    return CONTEXT + instruction
