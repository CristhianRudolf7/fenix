"""Regrade an immutable single-turn trace after a judge transport failure.

Does not start an agent or replay its actions. Run inside the pinned Gaia2 image.
The original result is preserved; the output is a separate audit record.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import time
import judge_daemon  # Configures transport compatibility, not grading rules.
from gaia2_cli.daemon.eventd import Gaia2EventDaemon
from gaia2_core.loader import ScenarioLoader
from gaia2_core.event_loop.processor import EventProcessor


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('scenario', 'trace', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--model', required=True)
    parser.add_argument('--reasoning-effort', required=True)
    parser.add_argument('--judge-url', required=True)
    args = parser.parse_args()
    args.output.mkdir(mode=0o700)
    loader = ScenarioLoader(str(args.scenario))
    processor = EventProcessor(events=loader.events, start_time=loader.start_time,
        duration=loader.duration, time_increment=loader.time_increment,
        app_name_to_class=loader.app_name_to_class)
    processor.build_turn_triggers()
    if processor.nb_turns != 1:
        raise SystemExit('Offline regrading currently supports single-turn traces only')
    shutil.copy2(args.trace / 'user_details.json', args.output / 'user_details.json')
    daemon = Gaia2EventDaemon(str(args.scenario), str(args.output),
        judge_model=args.model, judge_provider='openai', judge_base_url=args.judge_url,
        judge_api_key='evaluation-proxy-placeholder',
        judge_extra_body={'reasoning_effort': args.reasoning_effort, 'temperature': 1})
    daemon._processor = processor
    processor._judge = daemon._create_judge(loader, processor)
    events = args.trace / 'events.jsonl'
    entries = [json.loads(line) for line in events.read_text().splitlines() if line.strip()]
    daemon._add_events_to_processor(entries)
    started = time.monotonic()
    judgment = processor.judge_final_turn()
    result = {'judgment': judgment.model_dump(mode='json'), 'seconds': time.monotonic() - started,
        'model': args.model, 'reasoning_effort': args.reasoning_effort,
        'agent_rerun': False, 'reference_leaderboard_judge': False,
        'scenario_sha256': hashlib.sha256(args.scenario.read_bytes()).hexdigest(),
        'events_sha256': hashlib.sha256(events.read_bytes()).hexdigest()}
    (args.output / 'result.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
