import http.server,threading,subprocess,tempfile,pathlib,os,time,urllib.request,urllib.error,socket
repo=pathlib.Path(__file__).resolve().parent.parent
class Backend(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(404 if self.path=='/api/missing' else 200)
        self.send_header('Content-Type','text/event-stream' if self.path=='/api/events' else 'application/json')
        self.end_headers()
        if self.path=='/api/events':
            self.wfile.write(b'data: first\n\n');self.wfile.flush();time.sleep(2);self.wfile.write(b'data: last\n\n')
        else: self.wfile.write(b'{"api":true}')
    def log_message(self,*args): pass
backend=http.server.ThreadingHTTPServer(('127.0.0.1',0),Backend)
threading.Thread(target=backend.serve_forever,daemon=True).start()
with tempfile.TemporaryDirectory() as d:
    root=pathlib.Path(d);(root/'assets').mkdir();(root/'index.html').write_text('first-version');(root/'assets/worker.mjs').write_text('export {};');(root/'assets/test.wasm').write_bytes(b'\0asm')
    with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
    env={**os.environ,'FENIX_WEB_PORT':str(port),'FENIX_WEB_ROOT':d,'FENIX_API_UPSTREAM':f'127.0.0.1:{backend.server_port}'}
    with open(root/'caddy.log','w') as log:
        p=subprocess.Popen(['caddy','run','--config',str(repo/'packaging/linux/Caddyfile'),'--adapter','caddyfile'],env=env,stdout=log,stderr=log)
        try:
            base=f'http://127.0.0.1:{port}'
            for _ in range(40):
                try:urllib.request.urlopen(base,timeout=1);break
                except OSError:time.sleep(.1)
            for route in ['/chat','/documents','/projects/example']:
                r=urllib.request.urlopen(base+route);assert r.read()==b'first-version';assert r.headers['Cache-Control']=='no-cache';assert r.headers['X-Content-Type-Options']=='nosniff'
            for route in ['/assets/missing.js','/api/missing']:
                try:urllib.request.urlopen(base+route);raise AssertionError('Expected 404')
                except urllib.error.HTTPError as e:assert e.code==404
            for route,mime in [('worker.mjs','javascript'),('test.wasm','application/wasm')]:
                assert mime in urllib.request.urlopen(base+'/assets/'+route).headers['Content-Type']
            assert b'api' in urllib.request.urlopen(base+'/api/status').read()
            start=time.monotonic();r=urllib.request.urlopen(base+'/api/events');assert r.readline()==b'data: first\n';assert time.monotonic()-start<1.5;r.close()
            (root/'index.html').write_text('second-version');assert urllib.request.urlopen(base+'/chat').read()==b'second-version'
            print('PASS: SPA routes, asset/API 404s, MIME, security headers, unbuffered SSE, independent frontend update')
        finally:p.terminate();p.wait(timeout=10)
backend.shutdown()
