"""Unmodified Terminus2 with a per-trial recording endpoint."""

from harbor.agents.terminus_2.terminus_2 import Terminus2


class RecordedTerminus2(Terminus2):
    def __init__(self, logs_dir, **kwargs):
        self.audit_identifier = "terminus2-" + logs_dir.parent.name
        kwargs["api_base"] = "http://172.17.0.1:48931/" + self.audit_identifier + "/v1"
        super().__init__(logs_dir=logs_dir, **kwargs)

    async def run(self, instruction, environment, context):
        try:
            await super().run(instruction, environment, context)
        finally:
            context.metadata = context.metadata or {}
            context.metadata["api_trial"] = self.audit_identifier
