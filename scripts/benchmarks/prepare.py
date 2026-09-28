"""Materialize pinned official datasets without changing instructions or graders.

No model calls. Dataset artifacts are private evaluation inputs, not agent skills.
"""
import argparse
import gzip
import hashlib
import io
import json
import re
import tarfile
import tempfile
import urllib.request
from pathlib import Path, PurePosixPath


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'fenix-native-evaluation'})
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


def safe_path(name):
    path = PurePosixPath(name)
    if path.is_absolute() or '..' in path.parts or not path.parts:
        raise ValueError('Unsafe dataset artifact path')
    return path


def unpack(raw, output):
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
        for member in archive.getmembers():
            safe_path(member.name)
            if not (member.isfile() or member.isdir()):
                raise ValueError('Dataset archives must contain regular files or directories')
        archive.extractall(output, filter='data')


def prepare(benchmark, revision, output, index=0, split='execution'):
    if not re.fullmatch(r'[0-9a-f]{40}', revision):
        raise ValueError('Use an immutable 40-character upstream revision')
    if index < 0 or output.exists():
        raise ValueError('Index must be nonnegative and output must be new')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as temporary:
        stage = Path(temporary)
        manifest = {'benchmark': benchmark, 'dataset_revision': revision, 'index': index}
        if benchmark == 'terminalworld':
            base = f'https://huggingface.co/datasets/EuniAI/TerminalWorld/resolve/{revision}/'
            raw = fetch(base + 'data/verified.jsonl.gz')
            rows = [json.loads(line) for line in gzip.decompress(raw).splitlines() if line]
            row = rows[index]  # Preserve official verified-file ordering.
            artifact = str(safe_path(row['artifact_path']))
            unpack(fetch(base + artifact), stage / 'tasks')
            paths = list((stage / 'tasks').rglob('task.toml'))
            if len(paths) != 1:
                raise ValueError('Expected exactly one official task in archive')
            task = paths[0].parent
            manifest.update(task_id=row['task_id'], subset='verified', ordering='official JSONL order', source=base + artifact,
                            path=str(task.relative_to(stage)), dataset_size=len(rows))
        elif benchmark == 'terminal-bench-4.0':
            api = 'https://api.github.com/repos/harbor-framework/terminal-bench'
            tree = json.loads(fetch(f'{api}/git/trees/{revision}?recursive=1'))
            if tree.get('truncated'):
                raise ValueError('GitHub tree was truncated; cannot select reliably')
            paths = sorted(item['path'] for item in tree['tree'] if item['path'].startswith('tasks/') and item['path'].endswith('/task.toml'))
            prefix = paths[index].rsplit('/', 1)[0] + '/'
            base = f'https://raw.githubusercontent.com/harbor-framework/terminal-bench/{revision}/'
            for item in tree['tree']:
                if item['type'] == 'blob' and item['path'].startswith(prefix):
                    target = stage / str(safe_path(item['path']))
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(fetch(base + item['path']))
            manifest.update(task_id=prefix.split('/')[1], path=prefix.rstrip('/'), ordering='lexicographic task directory', dataset_size=len(paths), source=base)
        else:
            if split not in ('execution', 'search', 'ambiguity', 'adaptability', 'time'):
                raise ValueError('Unknown Gaia2 CLI split')
            import pyarrow.parquet as parquet
            base = f'https://huggingface.co/datasets/meta-agents-research-environments/gaia2-cli/resolve/{revision}/'
            raw = fetch(base + f'data/{split}-test-00000-of-00001.parquet')
            rows = sorted(parquet.read_table(io.BytesIO(raw)).to_pylist(), key=lambda row: row['scenario_id'])
            row = rows[index]
            target = stage / 'scenario.json'
            target.write_text(row['scenario'])
            manifest.update(task_id=row['scenario_id'], split=split, path='scenario.json', ordering='lexicographic scenario_id within split', dataset_size=len(rows), source=base)
        manifest['files_sha256'] = {str(p.relative_to(stage)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(stage.rglob('*')) if p.is_file()}
        (stage / 'selection.json').write_text(json.dumps(manifest, indent=2) + '\n')
        stage.rename(output)
    return manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('benchmark', choices=['terminalworld', 'terminal-bench-4.0', 'gaia2-cli'])
    parser.add_argument('--revision', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--index', type=int, default=0)
    parser.add_argument('--split', default='execution')
    args = parser.parse_args()
    print(json.dumps(prepare(args.benchmark, args.revision, args.output, args.index, args.split), indent=2))
