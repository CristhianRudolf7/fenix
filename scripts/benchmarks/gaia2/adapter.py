"""Gaia2 transport for the native Rust evaluator; never executes an LLM loop."""
import asyncio
import json
import os
from pathlib import Path

from gaia2_adapter_base import AdapterState, create_client_handler, run_adapter, write_aui_event, handle_notify, http_response
from gaia2_cli.daemon.cli_executor import execute_cli_action

SOCKET = '/tmp/fenix-evaluation.sock'
state = AdapterState()
worker = None
connected = False
sequence = 0


async def on_worker(reader, writer):
    global worker, connected
    if worker is not None:
        writer.close()
        return
    worker = writer
    try:
        async for raw in reader:
            message = json.loads(raw)
            if message['type'] == 'ready':
                connected = True
            elif message['type'] == 'response':
                if message['state'] in {'final', 'error'}:
                    # A failed task still has an outcome for the external judge.
                    # Preserve the error state; do not leave its turn open forever.
                    write_aui_event('send_message_to_user', message['message'])
                state.buffer_and_broadcast(message)
    finally:
        connected = False
        worker = None
        writer.close()


async def connect():
    Path(SOCKET).unlink(missing_ok=True)
    server = await asyncio.start_unix_server(on_worker, SOCKET)
    os.chmod(SOCKET, 0o660)
    asyncio.create_task(server.serve_forever())


async def notify(text, notification=False):
    global sequence
    if not connected:
        raise ConnectionError('Native Fenix evaluator is not ready')
    sequence += 1
    run_id = f'gaia2-{sequence}'
    worker.write((json.dumps({'type': 'message', 'run_id': run_id, 'text': text, 'notification': notification}) + '\n').encode())
    await worker.drain()
    return {'run_id': run_id}


async def execute(app, action, args, event_id):
    return execute_cli_action(app, action, args, event_id, state_dir='/var/gaia2/state')


async def extra_route(method, route, parsed, reader, writer, body):
    if method == 'POST' and route == '/evaluation/stop':
        if connected:
            worker.write(b'{"type":"stop"}\n')
            await worker.drain()
        http_response(writer, 200, {'ok': True})
        await writer.drain()
        writer.close()
        return True
    if method != 'POST' or route != '/send_notifications':
        return False
    await handle_notify(writer, body, lambda text: notify(text, notification=True), lambda: connected,
                        lambda text: write_aui_event('send_message_to_agent', text))
    await writer.drain()
    writer.close()
    return True


async def main():
    handler = create_client_handler(
        state=state, send_message=notify, is_connected=lambda: connected,
        get_health_info=lambda: {'backend': 'fenix-native'},
        on_notify_sent=lambda text: write_aui_event('send_message_to_agent', text),
        execute_action=execute, extra_routes=extra_route,
    )
    await run_adapter(state, handler, connect, backend_name='Fenix native')


if __name__ == '__main__':
    asyncio.run(main())
