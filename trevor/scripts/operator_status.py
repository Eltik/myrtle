#!/usr/bin/env python3
"""Playable operators and whether the story shows them dying, with evidence (P3 fact table).

Playable = character_table rows that are not TOKEN or TRAP and not IsNotObtainable (407).
Candidates: sentences of the P2 story summaries and the P3 dossiers that name the operator next to
a death word; the judge model keeps a sentence only if it states that this operator dies. Status is
"dies in the story" with evidence, or "no death found in the summaries", never "alive": absence of
evidence here is not evidence of survival. Writes artifacts/entities/operator_status.summary.jsonl.
Needs a llama-server on $SERVER.


Refuted as a served table (2026-09-27: 19 flagged, at most about 5 defensible). Superseded by
scripts/deaths.py, which reads the script; kept as the record of the refutation. The wiki cannot verify
it either: only 32 of 407 operator Story pages carry a status, none "Deceased", and an operator page
describes the operator (Civilight Eterna "Active") rather than the person (Theresa "Deceased").
"""
import json, os, re, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import ROOT

SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
DEATH = re.compile(r"\b(die[sd]?|dying|death|dead|killed|kills|perish(?:ed|es)?|slain|sacrific(?:ed|es)|passed away|lost (?:his|her|their) life)\b", re.I)
# v1 asked "does the named character die?" per (operator, sentence) and the judge answered true for
# anyone's death in the sentence: 50 operators "died", Skadi because Ulpianus did. Now the model names
# who dies in each sentence and the match to an operator is done in code.
SYS = ("You read one sentence from a summary of an Arknights story. List the characters the sentence states die or have "
       "died, by name exactly as written in the sentence. Leave out anyone only threatened, dying but not stated dead, "
       "presumed or faked dead, or killed only in a vision. Output a JSON list of names, or [] if none.")
# A leading title is dropped before the match ("Captain Ulpianus" is Ulpianus); a given name is not, so
# "Maria Nearl" never matches the operator Nearl.
TITLE = re.compile(r"^(?:(?:Captain|Lord|Lady|Duke|Duchess|Sir|Miss|Mr\.|Mrs\.|Ms\.|Dr\.|Master|King|Queen|Saint|General|"
                   r"Commander|Professor|Grand Knight|High Inquisitor|Inquisitor|Old|Young)\s+)+")
G_LIST = 'root ::= "[" ( str ( ", " str )* )? "]"\nstr ::= "\\"" [^"\\n]{1,60} "\\""\n'



def dead_in(sent):
    body = json.dumps(common.chat_body(SYS, sent, 80, grammar=G_LIST)).encode()
    # Its own retry: after the fourth failure it raises 'judge unreachable', not the last error.
    for attempt in range(4):
        try:
            return json.loads(common.content(common.post(common.chat_url(SERVER), body, 300)))
        except Exception:
            time.sleep(5 * (attempt + 1))
    raise RuntimeError('judge unreachable')


def main():
    ops = common.playable_operators()
    docs = [(s['storyId'], s['summary']) for s in map(json.loads, open(os.path.join(ROOT, 'artifacts', 'p2', 'stories.jsonl')))]
    docs += [('dossier:' + d['name'], d['dossier']) for d in map(json.loads, open(os.path.join(ROOT, 'artifacts', 'dossiers', 'dossiers.jsonl')))]
    sents = [(src, x.strip()) for src, t in docs for x in re.split(r'(?<=[.!?])\s+', t) if DEATH.search(x)]
    out = open(os.path.join(ROOT, 'artifacts', 'entities', 'operator_status.summary.jsonl'), 'w')
    n_cand = n_dead = 0; cache = {}
    for cid, name in ops:
        pat = re.compile(r"(?<![\w'])" + re.escape(name) + r"(?![\w'])")
        cands = [(src, x) for src, x in sents if pat.search(x)][:8]
        n_cand += len(cands)
        ev = []
        for src, x in cands:
            if x not in cache:
                cache[x] = dead_in(x)
            if name in {TITLE.sub('', n) for n in cache[x]} | set(cache[x]):
                ev.append({'source': src, 'sentence': x})
        status = 'dies in the story' if ev else 'no death found in the summaries'
        n_dead += bool(ev)
        out.write(json.dumps({'name': name, 'charId': cid, 'status': status, 'evidence': ev, 'candidates': len(cands)}, ensure_ascii=False) + '\n')
    out.close()
    print(f'operators {len(ops)}; candidate sentences {n_cand}; with a death stated {n_dead}')


if __name__ == '__main__':
    main()
