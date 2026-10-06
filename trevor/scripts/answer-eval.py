#!/usr/bin/env python3
"""Judge `ask` answers on a gold set: correctness, abstention, faithfulness, citations.

Needs a llama-server running the JUDGE model on $SERVER. Reads the gold set and the answers
written by `ask --batch`, writes <answers>.judged.jsonl (resumable) and prints totals.
  correct   answerable items: does the answer answer the question, consistent with the gold
            evidence (the item's evidence quotes and gold chunk text)? binary verdict.
  abstain   unanswerable items: does the answer decline (says it could not find it)? A fixed
            phrase list first, the judge for the rest.
  faithful  each answer sentence (citations stripped) against the passages the model was given;
            unsupported sentences are the fabrication rate.
  cited     answerable items: do the cited chunks include a gold chunk (line-span overlap)?
"""
import json, os, re, sys, time, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SERVER = os.environ.get('SERVER', 'http://127.0.0.1:8081')
CORPUS = os.path.join(ROOT, os.environ.get('CORPUS', 'artifacts/p3a'))
G_BOOL = 'root ::= "true" | "false"\n'
CORRECT_SYS = ("You grade an answer to a question about the Arknights story. You get the question, reference evidence "
               "from the story, and the answer. Answer true if the answer gives the correct answer to the question, "
               "consistent with the reference evidence, even if worded differently or with extra detail. Answer false "
               "if it is wrong, contradicts the evidence, misses the point of the question, or says it cannot find the "
               "answer. Answer true or false only.")
ABSTAIN_SYS = ("Does this answer decline to answer, saying the information could not be found or is not in the text? "
               "Answer true or false only.")
SUPPORT_SYS = ("You check one sentence of an answer against the passages it was based on. Answer true only if the "
               "passages state or clearly imply it. Answer false otherwise. Answer true or false only.")
DECLINE = re.compile(r"(could not find|couldn't find|cannot find|can't find|not (?:mentioned|stated|found|in the)|"
                     r"no information|does not (?:say|state|mention|contain)|doesn't (?:say|state|mention))", re.I)
ABBR = r'(?<!\bMr\.)(?<!\bMrs\.)(?<!\bMs\.)(?<!\bDr\.)(?<!\bSt\.)(?<!\bMt\.)(?<!\bNo\.)'


def verdict(system, user):
    body = json.dumps({'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': user}],
                       'temperature': 0, 'seed': 1, 'max_tokens': 3, 'grammar': G_BOOL}).encode()
    err = None
    for attempt in range(4):
        try:
            req = urllib.request.Request(f'{SERVER}/v1/chat/completions', body, {'content-type': 'application/json'})
            with urllib.request.urlopen(req, timeout=600) as r:
                return json.load(r)['choices'][0]['message']['content'].strip() == 'true'
        except Exception as e:
            err = e; time.sleep(5 * (attempt + 1))
    raise err


def sentences(t):
    t = re.sub(r'\s*\[[\d,;\s]+\]', '', ' '.join(t.split()))
    return [x.strip() for x in re.split(ABBR + r'(?<=[.!?])\s+(?=[A-Z"\'*])', t) if len(x.strip()) > 20]


def main(gold_path, ans_path):
    chunks = {}
    for l in open(os.path.join(CORPUS, 'chunks.jsonl')):
        r = json.loads(l); chunks[r['chunkId']] = r
    # The topic tool's summary is passage 'topic:<name>' (ask --router model); the judge sees it as ask did, citations stripped.
    tp = os.path.join(ROOT, 'artifacts', 'topics', 'topics.jsonl')
    for t in (map(json.loads, open(tp)) if os.path.exists(tp) else []):
        chunks[f"topic:{t['topic']}"] = {'text': re.sub(r'\s*\[[\d,;\s]+\]', '', t.get('summaryV2') or t.get('summary') or '')}  # summaryV2: --lore v2's text
    # A new dossier (`ask --lore v2`, 2026-10-02) is passage 'dossier:<name>', shown as ask shows it.
    dp = os.path.join(ROOT, 'artifacts', 'dossiers', 'dossiers.jsonl')
    for d in (map(json.loads, open(dp)) if os.path.exists(dp) else []):
        aka = [x for x in d.get('names') or [] if x != d['name']]
        head = f"Character profile: {d['name']}" + (f" (also known as {', '.join(aka)})" if aka else '')
        chunks[f"dossier:{d['name']}"] = {'text': f"{head}\n{d['dossier']}"}
    by_story = {}
    for c in chunks.values():
        if 'storyId' in c:
            by_story.setdefault(c['storyId'], []).append(c)
    gold = {json.loads(l)['qid']: json.loads(l) for l in open(gold_path)}
    answers = [json.loads(l) for l in open(ans_path)]
    out_path = ans_path.replace('.jsonl', '.judged.jsonl')
    done = {json.loads(l)['qid'] for l in open(out_path)} if os.path.exists(out_path) else set()
    print(f'judge: {len(answers) - len(done)} to do', flush=True)
    for a in answers:
        if a['qid'] in done:
            continue
        g = gold[a['qid']]
        answerable = g['stratum'] != 'unanswerable'
        gold_ids = set()
        for an in g.get('anchors', []):
            for c in by_story.get(an['story_id'], []):
                if c['lineStart'] <= an['line_end'] and c['lineEnd'] >= an['line_start']:
                    gold_ids.add(c['chunkId'])
        row = {'qid': a['qid'], 'stratum': g['stratum'], 'answerable': answerable}
        # A decline is an answer that opens by declining; "the text does not state X" inside a real
        # answer is not one (the first version counted it and forced such answers to wrong).
        first = re.split(r'(?<=[.!?])\s+', a['answer'].strip(), maxsplit=1)[0]
        declined = bool(DECLINE.search(first)) and verdict(ABSTAIN_SYS, a['answer'])
        row['declined'] = declined
        if answerable:
            ev = '\n'.join(g.get('evidence') or []) + '\n---\n' + '\n---\n'.join(chunks[i]['text'] for i in sorted(gold_ids) if i in chunks)
            row['correct'] = (not declined) and verdict(CORRECT_SYS, f"QUESTION: {g['question']}\n\nREFERENCE EVIDENCE:\n{ev}\n\nANSWER: {a['answer']}")
            row['citesGold'] = bool(gold_ids & set(a['cited']))
            row['goldInPassages'] = bool(gold_ids & set(a['passages']))
        gen = a.get('generated') or {}  # --lore v2's new dossier and game-data passages, as ask gave them
        given = '\n---\n'.join(gen[i] if i in gen else chunks[i]['text'] for i in a['passages'] if i in chunks or i in gen)
        sents = [] if declined else sentences(a['answer'])
        sup = [verdict(SUPPORT_SYS, f'PASSAGES:\n{given}\n\nSENTENCE: {s}') for s in sents]
        row.update({'sentences': len(sents), 'supported': sum(sup), 'unsupported': [s for s, v in zip(sents, sup) if not v],
                    'cited': len(a['cited']), 'invalidCitations': a['invalidCitations']})
        with open(out_path, 'a') as f:
            f.write(json.dumps(row, ensure_ascii=False) + '\n')
    R = [json.loads(l) for l in open(out_path)]
    A = [r for r in R if r['answerable']]; U = [r for r in R if not r['answerable']]
    n = len(A)
    print(f"answerable {n}: correct {sum(r['correct'] for r in A)} ({sum(r['correct'] for r in A) / max(n, 1):.3f}); "
          f"declined {sum(r['declined'] for r in A)}; gold chunk among passages {sum(r['goldInPassages'] for r in A)}; "
          f"cites a gold chunk {sum(r['citesGold'] for r in A)}")
    print(f"unanswerable {len(U)}: declined {sum(r['declined'] for r in U)}")
    s = sum(r['supported'] for r in R); t = sum(r['sentences'] for r in R)
    print(f"faithfulness: {s}/{t} answer sentences supported by the given passages = {s / max(t, 1):.3f}; "
          f"invalid citations {sum(r['invalidCitations'] for r in R)}")


if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
