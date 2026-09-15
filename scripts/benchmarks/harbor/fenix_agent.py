"""Harbor adapter: all reasoning and tool execution belong to Fenix."""

import json
import os
from pathlib import Path

from harbor.agents.base import BaseAgent
from harbor.models.agent.context import AgentContext


class FenixAgent(BaseAgent):
    api_prefix = "fenix"

    @staticmethod
    def name():
        return "fenix"

    def version(self):
        return "0.3.0-memory-working-tree"

    async def setup(self, environment):
        self.root = Path(os.environ["FENIX_AUDIT_ROOT"])
        await environment.upload_dir(self.root / "bundle", "/opt/fenix-audit")
        runner = Path(__file__).with_name("container_runner.py")
        await environment.upload_file(runner, "/opt/fenix-audit/container_runner.py")
        result = await environment.exec(
            "command -v python3 >/dev/null || (apt-get update -qq && apt-get install -y -qq python3)",
            timeout_sec=300, user="root")
        if result.return_code != 0:
            raise RuntimeError("Cannot install the benchmark HTTP controller: " + str(result.stderr))
        result = await environment.exec(
            "chmod +x /opt/fenix-audit/fenix /opt/fenix-audit/ld-linux-x86-64.so.2 && "
            "/opt/fenix-audit/ld-linux-x86-64.so.2 --library-path /opt/fenix-audit /opt/fenix-audit/fenix --help",
            timeout_sec=30)
        if result.return_code != 0:
            raise RuntimeError("Fenix binary preflight failed: " + str(result.stderr))

    async def run(self, instruction, environment, context: AgentContext):
        identifier = self.api_prefix + "-" + self.logs_dir.parent.name
        config = {"instruction": instruction,
                  "api_base": "http://172.17.0.1:48931/" + identifier + "/v1"}
        target = self.logs_dir / "run.json"
        target.write_text(json.dumps(config))
        await environment.upload_file(target, "/opt/fenix-audit/run.json")
        context.metadata = {"api_trial": identifier, "engine": "unmodified Fenix HTTP runtime"}
        returned = False
        try:
            result = await environment.exec("python3 /opt/fenix-audit/container_runner.py")
            returned = True
            (self.logs_dir / "controller-output.json").write_text(json.dumps({
                "return_code": result.return_code, "stdout": result.stdout, "stderr": result.stderr}))
            if result.return_code != 0:
                raise RuntimeError("Fenix controller failed; see controller-output.json")
        finally:
            if not returned:
                # Cancelling a local docker-exec client alone can leave the
                # remote controller running. End its inference phase explicitly.
                await environment.exec(
                    "python3 -c \"import os,signal; from pathlib import Path; "
                    "p=Path('/opt/fenix-audit/controller.pid'); "
                    "os.kill(int(p.read_text()),signal.SIGTERM) if p.exists() else None\"",
                    timeout_sec=30)
            usage = {"prompt_tokens": 0, "completion_tokens": 0, "cached_tokens": 0, "cost": 0.0}
            for path in (self.root / "api" / identifier).glob("*.metadata.json"):
                item = json.loads(path.read_text()).get("usage") or {}
                usage["prompt_tokens"] += item.get("prompt_tokens", 0)
                usage["completion_tokens"] += item.get("completion_tokens", 0)
                usage["cached_tokens"] += (item.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
                usage["cost"] += item.get("cost", 0)
            context.n_input_tokens = usage["prompt_tokens"]
            context.n_output_tokens = usage["completion_tokens"]
            context.n_cache_tokens = usage["cached_tokens"]
            context.cost_usd = usage["cost"]


class FenixLargeSummaryAgent(FenixAgent):
    """Diagnostic only: the proxy raises the internal summary output ceiling."""

    api_prefix = "fenix-cap"

    @staticmethod
    def name():
        return "fenix-summary-16384"
