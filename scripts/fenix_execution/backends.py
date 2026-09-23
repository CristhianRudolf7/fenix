"""Explicit isolation selection. Never downgrade after a startup failure."""
from fenix_isolation import PolicyError


def backend(config):
    kind = config.get("backend", "gvisor")
    if kind == "gvisor":
        from .gvisor import GVisor
        return GVisor(config)
    if kind == "microvm":
        from .microvm import MicroVM
        return MicroVM(config)
    if kind == "oci":
        from .docker import Docker
        return Docker(config)
    raise PolicyError("Unknown execution backend; no fallback is permitted")
