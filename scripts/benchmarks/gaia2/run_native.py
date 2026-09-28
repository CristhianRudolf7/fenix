"""Run Fenix with the official Gaia2-CLI daemon and judge (one attempt).

Set PYTHONPATH to the pinned upstream gaia2-cli/{runner,cli,core} directories.
Only the container transport and native report collection are customized.
"""
import argparse
import json
import logging
import time
from pathlib import Path
from gaia2_runner.launcher import LocalLauncher
from gaia2_runner.runner import ContainerRunner


class NativeLauncher(LocalLauncher):
    def __init__(self, output, model):
        super().__init__('docker')
        self.output = output
        self.model = model

    def _should_publish_adapter_port(self, **kwargs):
        return True

    def launch(self, *args, **kwargs):
        kwargs['network'] = 'bridge'
        kwargs.update(provider='openai', model=self.model, api_key='evaluation-proxy-placeholder')
        return super().launch(*args, **kwargs)

    def stop(self, container_id):
        # Stop inference and preserve the native trace before upstream removes
        # the container. The official judge has already produced its result.
        try:
            try:
                self.exec(container_id, ['python3', '-c',
                    "import os,urllib.request; urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:'+os.environ.get('GAIA2_ADAPTER_PORT','8090')+'/evaluation/stop',data=b'{}'),timeout=5).read()"])
            except Exception:
                self.exec(container_id, ['sh', '-c', 'kill -TERM "$(cat /tmp/fenix-native.pid)"'])
            for _ in range(100):
                try:
                    ready = self.exec(container_id, ['sh', '-c', 'test -f /home/agent/native/ready.json && echo ready || true']).strip() == 'ready'
                except Exception:
                    # The entrypoint may already have exited after native export;
                    # docker cp also works against a stopped container.
                    break
                if ready:
                    break
                time.sleep(.1)
            self.copy_from(container_id, '/home/agent/native', str(self.output / 'native'))
            self.copy_from(container_id, '/tmp/fenix-native.log', str(self.output / 'native.log'))
        except Exception as error:
            (self.output / 'native-collection-error.txt').write_text(str(error))
        finally:
            super().stop(container_id)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--selection', type=Path, required=True)
    parser.add_argument('--template', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--image', required=True)
    parser.add_argument('--judge-url', required=True)
    parser.add_argument('--port', type=int, default=48983)
    parser.add_argument('--timeout', type=int, default=900)
    args = parser.parse_args()
    args.output.mkdir(mode=0o700)
    selected = json.loads(args.selection.read_text())
    config = json.loads(args.template.read_text())
    config.update(benchmark='gaia2-cli', dataset_revision=selected['dataset_revision'], task_name=selected['task_id'])
    config['settings']['workspace'] = '/home/agent'
    config['settings']['base_url'] = config['settings']['base_url'].replace('{trial}', 'gaia2-agent')
    template = args.output / 'evaluation.json'
    template.write_text(json.dumps(config, indent=2))
    model = config['settings']['model']
    environment = {'PROVIDER': 'openai', 'MODEL': model, 'API_KEY': 'evaluation-proxy-placeholder',
        'GAIA2_JUDGE_MODEL': model, 'GAIA2_JUDGE_PROVIDER': 'openai',
        'GAIA2_JUDGE_BASE_URL': args.judge_url, 'GAIA2_JUDGE_API_KEY': 'evaluation-proxy-placeholder',
        'GAIA2_JUDGE_EXTRA_BODY': json.dumps({'reasoning_effort': config['settings']['reasoning_effort'], 'temperature': 1}),
        'GAIA2_JUDGE_FINAL_TURN': '1'}
    started = time.monotonic()
    runner = ContainerRunner(NativeLauncher(args.output, model), args.image, adapter_port=args.port)
    result = runner.run_scenario(str((args.selection.parent / selected['path']).resolve()),
        response_timeout=args.timeout, health_timeout=180, container_env=environment,
        extra_volumes=(str(template.resolve()) + ':/opt/fenix-evaluation.json:ro',), output_dir=str(args.output))
    result.update(wall_seconds=time.monotonic() - started, dataset_revision=selected['dataset_revision'],
        model=model, reasoning_effort=config['settings']['reasoning_effort'], judge_model=model,
        judge_protocol='official gaia2-cli, configurable Luna low judge; not reference leaderboard judge', attempts=1)
    (args.output / 'result.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)
    main()
