"""Harbor transport for `fenix eval`; no Python runtime required in task images."""
import asyncio
import json
import os
import shlex
from pathlib import Path

from evaluation_context import contextualize

from harbor.agents.base import BaseAgent
from harbor.models.agent.context import AgentContext


async def environment_workspace(environment):
    result = await environment.exec("pwd")
    workspace = result.stdout.strip()
    if result.return_code != 0 or not workspace.startswith("/") or "\n" in workspace:
        raise RuntimeError("Cannot determine the official task working directory")
    return workspace


async def wait_for_report(environment):
    while True:
        result = await environment.exec("test -f /logs/agent/native/ready.json")
        if result.return_code == 0:
            return
        alive = await environment.exec('kill -0 "$(cat /opt/fenix-eval/pid)"')
        if alive.return_code != 0:
            # The evaluator may atomically finish between the first check and
            # the process probe. A final ready marker wins over process exit.
            result = await environment.exec("test -f /logs/agent/native/ready.json")
            if result.return_code == 0:
                return
            raise RuntimeError("Native evaluator exited before reporting; inspect native-server.log")
        await asyncio.sleep(0.5)


def trial_config(template, workspace, trial):
    """Copy the template, binding the workspace and each role's `{trial}` endpoint."""
    config = json.loads(json.dumps(template))
    config["settings"]["workspace"] = workspace
    for role in (config["settings"], config.get("fast") or {}):
        if role.get("base_url"):
            role["base_url"] = role["base_url"].replace("{trial}", trial)
    return config


class NativeFenixAgent(BaseAgent):
    @staticmethod
    def name():
        return "fenix-native"

    def version(self):
        return "native-evaluation-v1"

    async def setup(self, environment):
        self.bundle = Path(os.environ["FENIX_EVAL_BUNDLE"])
        self.config = json.loads(Path(os.environ["FENIX_EVAL_CONFIG"]).read_text())
        self.workspace = await environment_workspace(environment)
        await environment.upload_dir(self.bundle, "/opt/fenix-eval")
        # Use a recording proxy with a disposable credential. Do not put the
        # provider's real key in an image where the evaluated agent runs shell.
        key = self.logs_dir / "proxy-key"
        key.write_text("evaluation-proxy-placeholder")
        key.chmod(0o600)
        await environment.upload_file(key, "/opt/fenix-eval/proxy-key")
        key.unlink()

    async def run(self, instruction, environment, context: AgentContext):
        config = trial_config(self.config, self.workspace, "native-" + self.logs_dir.parent.name)
        config.update(instruction=contextualize(instruction), api_key_file="/opt/fenix-eval/proxy-key",
                      task_name=self.logs_dir.parent.name)
        source = self.logs_dir / "native-config.json"
        source.write_text(json.dumps(config))
        await environment.upload_file(source, "/opt/fenix-eval/config.json")
        binary = "/opt/fenix-eval/fenix"
        if (self.bundle / "ld-linux-x86-64.so.2").exists():
            binary = "/opt/fenix-eval/ld-linux-x86-64.so.2 --library-path /opt/fenix-eval " + binary
        command = (binary + " eval /opt/fenix-eval/config.json /logs/agent/native --keep-services")
        launched = await environment.exec(
            "nohup sh -c " + shlex.quote("exec " + command) +
            " >/logs/agent/native-server.log 2>&1 </dev/null & echo $! >/opt/fenix-eval/pid")
        if launched.return_code != 0:
            raise RuntimeError("Cannot launch native Fenix evaluator")
        ready = False
        try:
            await wait_for_report(environment)
            ready = True
            result = await environment.exec("cat /logs/agent/native/report.json")
            report = json.loads(result.stdout)
            usage = report["metrics"]
            context.n_input_tokens = usage["input_tokens"]
            context.n_output_tokens = usage["output_tokens"]
            context.metadata = {"native_report": "native/report.json", "usage_scope": report["usage_scope"],
                                "stop_reason": report["stop_reason"], "official_reward": "external verifier"}
        finally:
            if not ready:
                await environment.exec('test ! -f /opt/fenix-eval/pid || kill -TERM "$(cat /opt/fenix-eval/pid)"', timeout_sec=30)
