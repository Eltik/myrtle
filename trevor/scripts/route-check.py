#!/usr/bin/env python3
"""Routing regression set: which questions `ask` answers from a table and which go to retrieval.

The table routes (story listing, deaths, real names, game-data attributes) are keyword rules; two
wrong routes were found by hand on 2026-09-28 ("Which operators betrayed Rhodes Island?" listed all 84
Rhodes Island operators; "Ursus" filtered as place and race at once). eval/routes.jsonl holds each
question, the route it must take (table or retrieval) and text a table answer must contain. Runs
`ask --route-only`. Exit 1 on any failure.

`--router keywords` (the default here, no model, 20 s) is the keyword router `update.sh` has always
checked. `--router model` needs Gemma on :8081 (ask connects to it; start it first, or ask starts and
stops one per question, about 15 s each). ASK=path runs another ask binary (a frozen copy while the tree is rebuilt). `--router knn` and `--router hybrid` route each question by the
other labelled examples and their paraphrases (`--knn-holdout`), since every route-check question is also a
labelled example; hybrid takes the model router's cases and needs Gemma too.
"""
import argparse, json, os, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ap = argparse.ArgumentParser()
ap.add_argument('--router', default='keywords', choices=['keywords', 'model', 'knn', 'hybrid', 'off'])
ap.add_argument('extra', nargs='*', help='more ask flags, after --')
args = ap.parse_args()
flags = ['--router', args.router] + (['--knn-holdout'] if args.router in ('knn', 'hybrid') else []) + args.extra
fails = 0
# Cases with "routers" apply only to those routers (canon, topic and Storylines, 2026-09-29, which the keyword
# router never had); "tool" also checks the router's chosen tool (model and kNN print it as a ROUTE line).
cases = [c for c in (json.loads(l) for l in open(os.path.join(ROOT, 'eval', 'routes.jsonl')))
         if ('model' if args.router == 'hybrid' else args.router) in c.get('routers', ['keywords', 'model', 'knn', 'off'])]
for c in cases:
    out = subprocess.run([os.environ.get('ASK') or os.path.join(ROOT, 'target', 'release', 'ask'), '--route-only', *flags, c['q']], cwd=ROOT,
                         capture_output=True, text=True).stdout
    route = ''
    if out.startswith('ROUTE '):
        route, _, out = out.partition('\n')
        route = route[len('ROUTE '):]
    got = 'retrieval' if out.startswith('RETRIEVAL') else 'table'
    missing = [t for t in c.get('contains', []) if t not in out]
    tool = json.loads(route)['tool'] if route else None
    wrong_tool = 'tool' in c and tool != c['tool']
    ok = got == c['expect'] and not missing and not wrong_tool
    fails += not ok
    print(f"{'ok  ' if ok else 'FAIL'} {c['expect']:9} got {got:9} {c['q'][:70]}" + (f"  missing {missing}" if missing else '')
          + (f"  {route}" if route and not ok else ''))
print(f'route check ({args.router}): {len(cases) - fails}/{len(cases)} pass')
sys.exit(1 if fails else 0)
