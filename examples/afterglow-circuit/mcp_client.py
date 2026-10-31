"""Small auditable HTTP client. Audio is synthesized exclusively by CTG Studio."""
import json
from pathlib import Path
import requests

ROOT = Path(__file__).resolve().parent

class CTG:
    def __init__(self):
        self.session = requests.Session()
        self.headers = {'Accept': 'application/json, text/event-stream'}
        self.seq = 0
        self.audit = ROOT / 'mcp-call-audit.jsonl'
        self.rpc('initialize', {'protocolVersion':'2024-11-05','capabilities':{},'clientInfo':{'name':'Afterglow Circuit authored score','version':'1.0'}})
        self.rpc('notifications/initialized', {}, notification=True)

    def rpc(self, method, params, notification=False):
        self.seq += 1
        payload = {'jsonrpc':'2.0','method':method,'params':params}
        if not notification:
            payload['id'] = self.seq
        response = self.session.post('http://127.0.0.1:17416/mcp', headers=self.headers, json=payload, timeout=300)
        response.raise_for_status()
        if 'mcp-session-id' in response.headers:
            self.headers['mcp-session-id'] = response.headers['mcp-session-id']
        if notification:
            return None
        data = None
        if response.headers.get('content-type','').startswith('text/event-stream'):
            for line in response.text.splitlines():
                if line.startswith('data: {'):
                    data = json.loads(line[6:])
        else:
            data = response.json()
        if 'error' in data:
            raise RuntimeError(data)
        return data['result']

    def call(self, tool, **arguments):
        assert tool not in {'song_compose','composition_random','composition_preview'}
        result = self.rpc('tools/call', {'name':tool,'arguments':arguments})
        with self.audit.open('a',encoding='utf-8') as stream:
            stream.write(json.dumps({'tool':tool,'arguments':arguments,'result':result})+'\n')
        if result.get('isError'):
            raise RuntimeError((tool,result))
        content = result.get('structuredContent')
        if content is None:
            texts = [x['text'] for x in result['content'] if x['type']=='text']
            content = json.loads('\n'.join(texts))
        return content

if __name__ == '__main__':
    client=CTG()
    for name in ['reference_get','preset_list','song_get']:
        data=client.call(name)
        (ROOT/(name+'.json')).write_text(json.dumps(data,indent=2),encoding='utf-8')
        print(name, str(data)[:900])
