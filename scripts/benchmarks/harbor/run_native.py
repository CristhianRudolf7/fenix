"""Run one official Harbor task through Fenix, using a prepared selection manifest."""
import argparse
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('selection', 'template', 'bundle', 'output', 'harbor'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--override-cpus', type=int, help='Explicit local resource deviation; never silently cap official resources')
    args = parser.parse_args()
    if args.override_cpus is not None and args.override_cpus < 1:
        parser.error('--override-cpus must be positive')
    args.output.mkdir(mode=0o700)  # Never reuse another trial's state.
    selection = json.loads(args.selection.read_text())
    template = json.loads(args.template.read_text())
    template.update(benchmark=selection['benchmark'], dataset_revision=selection['dataset_revision'])
    config = args.output / 'evaluation.json'
    config.write_text(json.dumps(template, indent=2))
    job = {'job_name': 'fenix-' + selection['task_id'], 'jobs_dir': str(args.output.resolve() / 'jobs'),
        'n_concurrent_trials': 1, 'n_attempts': 1, 'retry': {'max_retries': 0},
        'environment': {'type': 'docker', 'delete': True},
        'agents': [{'model_name': template['settings']['provider'] + '/' + template['settings']['model'],
                    'import_path': 'native_fenix:NativeFenixAgent'}],
        'tasks': [{'path': str((args.selection.parent / selection['path']).resolve())}]}
    if args.override_cpus is not None:
        job['environment']['override_cpus'] = args.override_cpus
    source = args.output / 'job.json'
    source.write_text(json.dumps(job, indent=2))
    env = {k: v for k, v in os.environ.items() if not any(secret in k for secret in ('API_KEY', 'AUTH_TOKEN', 'ACCESS_TOKEN', 'CODEX_AUTH'))}
    env.update(FENIX_EVAL_BUNDLE=str(args.bundle.resolve()), FENIX_EVAL_CONFIG=str(config.resolve()),
               PYTHONPATH=str(Path(__file__).resolve().parent))
    raise SystemExit(subprocess.call([str(args.harbor.resolve()), 'run', '-c', str(source.resolve())], env=env))


if __name__ == '__main__':
    main()
