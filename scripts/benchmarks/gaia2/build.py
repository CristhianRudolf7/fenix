"""Build the native Fenix Gaia2 image against an explicitly pinned ARE checkout."""
import argparse
import shutil
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--upstream', type=Path, required=True)
    parser.add_argument('--revision', required=True)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--image', default='fenix-eval-gaia2:local')
    args = parser.parse_args()
    revision = subprocess.check_output(['git', '-C', str(args.upstream), 'rev-parse', 'HEAD'], text=True).strip()
    if revision != args.revision:
        raise SystemExit('Upstream checkout differs from pinned ARE revision')
    context = args.upstream / 'gaia2-cli'
    base = 'fenix-eval-gaia2-base:' + revision[:7]
    subprocess.run(['docker', 'build', '-f', 'cli/Dockerfile', '-t', base, '.'], cwd=context, check=True)
    adapter = context / 'fenix'
    adapter.mkdir(exist_ok=True)
    for name in ('adapter.py', 'entrypoint.py', 'judge_daemon.py', 'Dockerfile'):
        shutil.copy2(Path(__file__).parent / name, adapter / name)
    shutil.copytree(args.bundle, context / 'bundle', dirs_exist_ok=True)
    subprocess.run(['docker', 'build', '-f', 'fenix/Dockerfile', '--build-arg', 'GAIA2_BASE=' + base,
                    '-t', args.image, '.'], cwd=context, check=True)


if __name__ == '__main__':
    main()
