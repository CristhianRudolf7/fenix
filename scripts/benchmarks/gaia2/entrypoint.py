"""Privilege-separated setup around the unmodified Gaia2 CLI daemon and judge."""
import json
import os
import pwd
import signal
import subprocess
import time
from pathlib import Path
from datetime import datetime, timezone

children = []


def launch(args, user, env, log):
    account = pwd.getpwnam(user)
    groups = os.getgrouplist(user, account.pw_gid)
    handle = open(log, 'a')
    os.fchmod(handle.fileno(), 0o600)
    os.fchown(handle.fileno(), account.pw_uid, account.pw_gid)
    process = subprocess.Popen(args, user=account.pw_uid, group=account.pw_gid,
        extra_groups=groups, env=env, stdout=handle, stderr=subprocess.STDOUT)
    handle.close()
    children.append(process)
    return process


def main():
    scenario = os.environ.get('GAIA2_SCENARIO', '/var/gaia2/custom_scenario.json')
    state = Path('/var/gaia2/state')
    subprocess.run(['gaia2-init', '--scenario', scenario, '--state-dir', str(state),
        '--fs-backing-dir', '/opt/gaia2_filesystem'], check=True)
    subprocess.run(['chown', '-R', 'gaia2:benchmark-transport', str(state)], check=True)
    subprocess.run(['chmod', '-R', 'go-rwx', str(state)], check=True)
    from gaia2_cli.app_registry import resolve_scenario_tools
    for command in resolve_scenario_tools(scenario).get('remove', []):
        Path('/home/agent/bin', command).unlink(missing_ok=True)
    subprocess.run(['python3', '/opt/render_agent_prompt.py', '--scenario', scenario,
        '--template', '/opt/AGENTS_TEMPLATE.md', '--exec-tool', 'command_run',
        '--output-prompt', '/home/agent/AGENTS.md'], check=True)
    definition = json.loads(Path(scenario).read_text()).get('metadata', {}).get('definition', {})
    clock = Path('/tmp/faketime.rc')
    clock.write_text(datetime.fromtimestamp(float(definition.get('start_time', 0)), timezone.utc).strftime('%Y-%m-%d %H:%M:%S'))
    os.chown(clock, pwd.getpwnam('gaia2').pw_uid, pwd.getpwnam('gaia2').pw_gid)
    clock.chmod(0o644)
    env = dict(os.environ, GAIA2_STATE_DIR=str(state), PYTHONUNBUFFERED='1')
    # Exercise the actual judge library before starting expensive agent work.
    subprocess.run(['python3', '/opt/judge_daemon.py', '--check'], env=env, check=True)
    port = env.get('GAIA2_ADAPTER_PORT', '8090')
    launch(['python3', '/opt/adapter.py'], 'gaia2', env, '/tmp/gaia2-adapter.log')
    deadline = time.monotonic() + 30
    while not Path('/tmp/fenix-evaluation.sock').exists():
        if time.monotonic() > deadline:
            raise RuntimeError('Evaluation socket did not start')
        time.sleep(0.1)
    config = json.loads(Path('/opt/fenix-evaluation.json').read_text())
    config.update(instruction=Path('/home/agent/AGENTS.md').read_text() + '\nFenix execution note: invoke the listed app commands with their absolute /home/agent/bin/<command> paths because command_run sanitizes PATH. The simulated UTC clock is available in /tmp/faketime.rc; use it instead of the host clock for scenario time. Complete execution through task_finish; the evaluator delivers its answer to the user.\n', interaction_socket='/tmp/fenix-evaluation.sock')
    Path('/home/agent/evaluation.json').write_text(json.dumps(config))
    Path('/home/agent/proxy-key').write_text('evaluation-proxy-placeholder')
    config['api_key_file'] = '/home/agent/proxy-key'
    Path('/home/agent/evaluation.json').write_text(json.dumps(config))
    subprocess.run(['chown', '-R', 'agent:agent', '/home/agent'], check=True)
    native_env = {k: v for k, v in env.items() if not any(secret in k for secret in ('KEY', 'TOKEN', 'PASSWORD'))}
    native_env.pop('HOME', None)
    native_env['PATH'] = '/home/agent/bin:/usr/local/bin:/usr/bin:/bin'
    binary = ['/opt/fenix/fenix']
    if Path('/opt/fenix/ld-linux-x86-64.so.2').exists():
        binary = ['/opt/fenix/ld-linux-x86-64.so.2', '--library-path', '/opt/fenix', *binary]
    native = launch([*binary, 'eval', '/home/agent/evaluation.json', '/home/agent/native'], 'agent', native_env, '/tmp/fenix-native.log')
    Path('/tmp/fenix-native.pid').write_text(str(native.pid))
    daemon = ['python3', '/opt/judge_daemon.py', '--scenario', scenario, '--state-dir', str(state),
        '--notify-url', f'http://127.0.0.1:{port}', '--poll-interval', '0.5', '--faketime-path', str(clock)]
    for key, flag in [('MODEL','model'),('PROVIDER','provider'),('BASE_URL','base-url'),('API_KEY','api-key'),('EXTRA_BODY','extra-body')]:
        if value := env.get('GAIA2_JUDGE_' + key):
            daemon += ['--judge-' + flag, value]
    launch(daemon, 'gaia2', env, '/tmp/gaia2-eventd.log')
    native.wait()


def stop(*_):
    for child in reversed(children):
        child.terminate()
    for child in reversed(children):
        try: child.wait(timeout=10)
        except subprocess.TimeoutExpired: child.kill()
    raise SystemExit(0)


if __name__ == '__main__':
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    try: main()
    finally: stop()
