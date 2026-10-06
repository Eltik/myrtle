import json, os, re, sys, time, urllib.request, collections, importlib.util
T='/Users/eltik/Documents/Coding/myrtle/trevor'  # eval tool: its T1 targets come from the wiki reference (eval/reference/reference.json)
sp=importlib.util.spec_from_file_location('c',f'{T}/scripts/chrono.py'); c=importlib.util.module_from_spec(sp); sp.loader.exec_module(c)
ref=c.reference(); anchors,_,rel=c.backbone(); names=c.group_names()
summ={g['groupId']:g['summary'] for g in c.read_jsonl(f'{T}/artifacts/p2/groups.jsonl') if g['method']=='C'}
gof={json.loads(l)['storyId']:json.loads(l)['groupId'] for l in open(f'{T}/artifacts/chunks.jsonl')}
dated=collections.defaultdict(list)
for e in c.read_jsonl(f'{T}/artifacts/chrono/events.jsonl'):
    if e['is_year'] and e['kind']=='story': dated[gof[e['source']]].append(e)
targets=json.load(open(f'{T}/artifacts/chrono/disagree.json'))+sorted(g for g,v in ref.items() if v[0]=='T1')
SYS=c.EXC_SYS+(" Think it through, then end with one line holding only a JSON object: "
               '{"setting": "storyline" | "earlier" | "later", "year": <year or null>, "reason": "<one sentence>"}.')
out=open('results.jsonl','a')
for g in dict.fromkeys(targets):
    sy=c.storyline_year(rel[g],anchors)
    lines='\n'.join(f"- {e['year']}: {e['event']} (\"{e['line'][:160]}\")" for e in dated[g]) or '(no dated lines)'
    user=(f'The ongoing storyline has reached about year {sy:.0f} around this event.\n\nEvent: {names.get(g,g)}\n'
          f'Summary: {summ.get(g,"(none)")}\n\nDated lines from its script:\n{lines}')
    body=json.dumps({'messages':[{'role':'system','content':SYS},{'role':'user','content':user}],'temperature':0,'seed':1,'max_tokens':3000}).encode()
    t0=time.time()
    with urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:8081/v1/chat/completions',body,{'content-type':'application/json'}),timeout=1800) as r:
        d=json.load(r)
    m=d['choices'][0]['message']; content=m.get('content') or ''; think=m.get('reasoning_content') or ''
    js=re.findall(r'\{[^{}]*"setting"[^{}]*\}',content)
    try: ans=json.loads(js[-1])
    except Exception: ans={'setting':'unparsed','year':None,'reason':content[-200:]}
    ans.update({'groupId':g,'storylineYear':round(sy,2),'ref':ref.get(g),'thinkChars':len(think),'seconds':round(time.time()-t0,1)})
    out.write(json.dumps(ans,ensure_ascii=False)+'\n'); out.flush()
    print(f"{g}: {ans['setting']} {ans.get('year')} ref {ref.get(g)} think {len(think)} chars {ans['seconds']}s | {str(ans.get('reason'))[:110]}",flush=True)
