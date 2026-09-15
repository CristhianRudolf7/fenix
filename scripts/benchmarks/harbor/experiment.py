"""Run frozen five-task native cohorts and summarize external verifier/billing evidence."""
import argparse
from collections import Counter
import hashlib
import json
import os
import subprocess
import time
import tomllib
from pathlib import Path


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def run(args):
    tasks = args.tasks.split(',')
    if len(tasks) != 5 or len(set(tasks)) != 5:
        raise ValueError('Each cohort requires five distinct task names')
    for task in tasks:
        if Path(task).name != task or not (args.dataset / task / 'task.toml').is_file():
            raise ValueError('Unknown task: ' + task)
    args.root.mkdir(parents=True, exist_ok=True)
    trial = args.root / (args.job + '.protocol.json')
    manifest = {'job': args.job, 'tasks': tasks, 'attempts': args.attempts,
                'binary_sha256': digest(args.bundle / 'fenix'),
                'configuration_sha256': digest(args.config),
                'git_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
                'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], text=True).strip()),
                'reward_source': 'official unmodified verifier',
                'model': 'nemotron-3-5-lightning-30b'}
    with trial.open('x') as stream:
        json.dump(manifest, stream, indent=2)
    config = {'job_name': args.job, 'jobs_dir': str(args.root / 'jobs'),
              'n_concurrent_trials': 2, 'n_attempts': args.attempts,
              'retry': {'max_retries': 0}, 'environment': {'type': 'docker', 'delete': True},
              'agents': [{'import_path': 'native_fenix:NativeFenixAgent',
                          'model_name': 'openai/nemotron-3-5-lightning-30b',
                          'override_setup_timeout_sec': 600}],
              'tasks': [{'path': str(args.dataset / task)} for task in tasks]}
    config_path = args.root / (args.job + '.harbor.json')
    config_path.write_text(json.dumps(config, indent=2))
    env = dict(os.environ, FENIX_EVAL_BUNDLE=str(args.bundle), FENIX_EVAL_CONFIG=str(args.config),
               PYTHONPATH=str(Path(__file__).resolve().parent), TMPDIR=str(args.root / 'tmp'))
    (args.root / 'tmp').mkdir(exist_ok=True)
    inventory = args.root / 'initial-images.json'
    if not inventory.exists():
        inventory.write_text(json.dumps(subprocess.check_output(['docker','image','ls','--format','{{.Repository}}:{{.Tag}}'],text=True).splitlines()))
    protected = set(json.loads(inventory.read_text()))
    cleaned = set()
    with (args.root / (args.job + '.log')).open('x') as log:
        process = subprocess.Popen([str(args.harbor), 'run', '-c', str(config_path)], env=env, stdout=log, stderr=subprocess.STDOUT)
        while process.poll() is None:
            time.sleep(5)
            finished = []
            for result_path in (args.root/'jobs'/args.job).glob('*/result.json'):
                try:
                    result = json.loads(result_path.read_text())
                    if result.get('finished_at'): finished.append(result['task_name'].split('/')[-1])
                except (OSError, ValueError):
                    continue
            for task in tasks:
                if finished.count(task) < args.attempts or task in cleaned: continue
                task_config = tomllib.loads((args.dataset/task/'task.toml').read_text())
                image = task_config.get('environment',{}).get('docker_image')
                if image and image not in protected:
                    # Never force removal: Docker refuses an image used by a container.
                    subprocess.run(['docker','image','rm',image],stdout=log,stderr=subprocess.STDOUT,check=False)
                cleaned.add(task)
    manifest['exit_code'] = process.returncode
    manifest['binary_unchanged'] = digest(args.bundle / 'fenix') == manifest['binary_sha256']
    manifest['configuration_unchanged'] = digest(args.config) == manifest['configuration_sha256']
    trial.write_text(json.dumps(manifest, indent=2))
    summarize(args.root)
    if not manifest['binary_unchanged'] or not manifest['configuration_unchanged']:
        raise RuntimeError('Trial input changed during evaluation')
    return process.returncode


def summarize(root):
    rows = []
    for path in sorted((root / 'jobs').glob('*/*/result.json')):
        result = json.loads(path.read_text())
        if not result.get('finished_at'):
            continue
        native_path = path.parent / 'agent/native/report.json'
        native = json.loads(native_path.read_text()) if native_path.exists() else {}
        events = [e for trace in native.get('traces', []) for e in trace.get('events', [])]
        calls = [json.loads(p.read_text()) for p in (root / 'api' / ('native-' + path.parent.name)).glob('*.metadata.json')]
        usages = [c['usage'] for c in calls if isinstance(c.get('usage'), dict)]
        def total(key):
            return sum(u[key] for u in usages if isinstance(u.get(key), (int, float)))
        reward = ((result.get('verifier_result') or {}).get('rewards') or {}).get('reward')
        rows.append({'job': path.parent.parent.name, 'task': result['task_name'].split('/')[-1],
                     'trial': path.parent.name, 'reward': reward,
                     'exception': (result.get('exception_info') or {}).get('exception_type'),
                     'native_seconds': native.get('elapsed_seconds'),
                     'native_metrics': native.get('metrics'), 'stop_reason': native.get('stop_reason'),
                     'events':dict(Counter(e['kind'] for e in events)),
                     'cached_input_tokens':sum((u.get('prompt_tokens_details') or {}).get('cached_tokens',0) for u in usages),
                     'calls': len(calls), 'input_tokens': total('prompt_tokens'),
                     'output_tokens': total('completion_tokens'),
                     'cost_usd': total('cost') if usages and all('cost' in u for u in usages) and len(usages)==len(calls) else None,
                     'missing_usage': len(calls)-len(usages),
                     'http_errors': sum(c.get('http_status') != 200 for c in calls),
                     'agent_execution': result.get('agent_execution'),
                     'verifier_execution': result.get('verifier_execution')})
    (root / 'results.json').write_text(json.dumps(rows, indent=2))
    print(json.dumps(rows, indent=2), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=lambda p: Path(p).resolve(), required=True)
    parser.add_argument('--summarize', action='store_true')
    parser.add_argument('--job')
    parser.add_argument('--tasks')
    parser.add_argument('--attempts', type=int, choices=[1,2], default=1)
    for name in ['dataset','bundle','config','harbor']:
        parser.add_argument('--'+name, type=lambda p: Path(p).resolve())
    args = parser.parse_args()
    if args.summarize:
        summarize(args.root)
    else:
        if not all(getattr(args, n) for n in ['job','tasks','dataset','bundle','config','harbor']):
            parser.error('Running requires job, tasks, dataset, bundle, config and harbor')
        if Path(args.job).name != args.job or args.job in ('.','..'):
            parser.error('job must be a directory name')
        raise SystemExit(run(args))
