#!/usr/bin/env python3
"""Operator design inspirations from the Arknights wiki's trivia pages: an EVAL REFERENCE only.

Written 2026-10-04 morning for Ian's ian11 and served for a few hours; since 2026-10-04 (Ian: "use the Wiki
as a *reference*") nothing Trevor serves or builds reads it. scripts/design_infer.py deduces design
inspirations from the game data and scores them against this table. For every operator in artifacts/entities/operator_attributes.jsonl, fetch arknights.wiki.gg
"<Name>/Trivia" through the MediaWiki parse API (redirects followed; a missing page is cached as such),
one request a second with a descriptive User-Agent, raw responses cached under eval/reference/wiki_cache/ and
never refetched (DESIGN_REFETCH=1 refetches). Then keep each trivia item sentence with a design cue
("based on", "inspired by", "motif", "modeled after", "reference to", "named after", ...) and take as its
basis the subjects linked in it (wiki and Wikipedia links, the link nearest the cue first), falling back to
the noun phrase after the cue. Writes eval/reference/design_basis.jsonl:
{operator, charId, basis, sentence, section, alters (the
other operators of its game alter group, char_meta_table SpCharGroups), url, fetched}, one row per sentence.

  python3 scripts/design_basis.py [fetch|extract|all]   (default all; extract needs no network)
"""
import html, json, os, re, sys, time, urllib.parse

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common
from common import EXCEL, ROOT

ATTR = os.path.join(ROOT, 'artifacts', 'entities', 'operator_attributes.jsonl')
CACHE = os.path.join(ROOT, 'eval', 'reference', 'wiki_cache')
OUT = os.path.join(ROOT, 'eval', 'reference', 'design_basis.jsonl')
META = os.path.join(EXCEL, 'char_meta_table.json')
UA = 'trevor-research/0.1 (local Arknights lore assistant; design-basis trivia, 1 req/s, cached)'


def page_name(name):
    """Wiki page title for an operator name: the game's quotes are dropped ('Justice Knight')."""
    return name.strip().strip("'\"").replace('’', "'")


def cache_path(page):
    return os.path.join(CACHE, urllib.parse.quote(page, safe='') + '.json')


def fetch(page):
    p = cache_path(page)
    if os.path.exists(p) and os.environ.get('DESIGN_REFETCH') != '1':
        return json.load(open(p)), False
    t = common.wiki_get_retry({'action': 'parse', 'page': page, 'prop': 'text', 'format': 'json', 'redirects': 1}, page, ua=UA)
    t['_fetched'] = time.strftime('%Y-%m-%d')
    json.dump(t, open(p, 'w'), ensure_ascii=False)
    return t, True


def operators():
    return [json.loads(l) for l in open(ATTR)]


def cmd_fetch():
    os.makedirs(CACHE, exist_ok=True)
    n_new = 0
    for op in operators():
        _, new = fetch(page_name(op['name']) + '/Trivia')
        if new:
            n_new += 1
            time.sleep(1)
    print(f'fetch: {len(operators())} operators, {n_new} fetched now', file=sys.stderr)


CUE = re.compile(r'\b(?:based (?:on|off)|inspired by|inspiration (?:from|for)|motifs?\b|modell?ed (?:after|on)|'
                 r'designed after|derived from|references?\b|alludes? to|allusion to|named after|homage to|'
                 r'nod to|takes? after|reminiscent of)', re.I)
LINK = re.compile(r'<a\s[^>]*?href="([^"]*)"[^>]*>(.*?)</a>', re.S)
TAG = re.compile(r'<(?:[^>"\']|"[^"]*"|\'[^\']*\')*>')
# "spider crabs (which Akafuyu is based on)": the subject comes before the cue.
PAREN = re.compile(r"(?:the |an? )?((?:[\w'-]+ ){0,3}[\w'-]+)\s*\((?:which|whom|who|that)\b[^()]*\b(?:based|inspired|modell?ed)\b[^()]*\)", re.I)
SKIP_LINK = re.compile(r'(?:/wiki/(?:File|Category|Template):|action=edit|#cite)', re.I)


def text_of(h):
    return re.sub(r'\s+', ' ', html.unescape(TAG.sub('', h))).strip()


def items(page_html):
    """(section, li html) for each top-level trivia list item; nested lists are kept inside their item."""
    out, section = [], ''
    for m in re.finditer(r'<h[23][^>]*>(.*?)</h[23]>|<li>(.*?)</li>', page_html, re.S):
        if m.group(1) is not None:
            section = text_of(re.sub(r'<span class="mw-editsection".*?</span></span>', '', m.group(1), flags=re.S))
        else:
            out.append((section, m.group(2)))
    return out


def sentences(li_html):
    """Split an item into sentences, keeping the html of each so its links stay attached."""
    parts, start = [], 0
    plain = li_html
    for m in re.finditer(r'(?<=[.!?])(?:</a>)?\s+(?=[A-Z<])', plain):
        parts.append(plain[start:m.end()])
        start = m.end()
    parts.append(plain[start:])
    return [p for p in parts if text_of(p)]


def basis_of(sent_html):
    """Linked subjects of a cued sentence, the link nearest the cue first; else the phrase after the cue."""
    t = text_of(sent_html)
    cue = CUE.search(t)
    links = []
    for m in LINK.finditer(sent_html):
        href, anchor = m.group(1), text_of(m.group(2))
        if not anchor or SKIP_LINK.search(href) or anchor.lower() in ('edit',):
            continue
        pos = len(text_of(sent_html[:m.start()]))
        links.append((abs(pos - cue.start()) if cue else 0, anchor))
    names = []
    pm = PAREN.search(t)
    if pm:
        before = [a for d, a in links if a.lower() in pm.group(1).lower() or pm.group(1).lower().endswith(a.lower())]
        names.append(before[0] if before else re.sub(r'^.*\bthat ', '', pm.group(1).strip()))
    for _, a in sorted(links, key=lambda x: x[0]):
        if a not in names:
            names.append(a)
    strong = cue and re.match(r'based|inspired|modell?ed|designed', cue.group(0), re.I)
    if cue and (not names or strong):
        tail = t[cue.end():]
        m = re.match(r'\s*(?:the fact that |the |an? )?([^,.;:()"]{2,60})', tail)
        if m and m.group(1).strip() not in names and not re.match(r'(?:her|his|their|its|this|that)\b', m.group(1)):
            names.append(m.group(1).strip())
    return names


def alter_groups():
    """charId -> the other charIds of its game alter group (char_meta_table SpCharGroups: Phantom and Tragodia)."""
    try:
        groups = json.load(open(META))['SpCharGroups']
    except (OSError, KeyError, ValueError):
        return {}
    out = {}
    for g in groups:
        ids = g['value']
        for c in ids:
            out[c] = [x for x in ids if x != c]
    return out


def cmd_extract():
    rows, with_page, with_basis = [], 0, set()
    ops = operators()
    name_of = {o['charId']: o['name'] for o in ops}
    alters = alter_groups()
    for op in ops:
        page = page_name(op['name']) + '/Trivia'
        p = cache_path(page)
        if not os.path.exists(p):
            continue
        t = json.load(open(p))
        if 'parse' not in t:
            continue
        with_page += 1
        title = t['parse'].get('title', page)
        url = 'https://arknights.wiki.gg/wiki/' + urllib.parse.quote(title.replace(' ', '_'), safe='/')
        # Outfit tooltips carry their description in data-* attributes, which the sentence splitter would cut into.
        page_html = re.sub(r'\sdata-[\w-]+="[^"]*"', '', t['parse']['text']['*'])
        for section, li in items(page_html):
            if re.search(r'distinction|miscellaneous|voice|cv\b', section, re.I):
                continue
            for s in sentences(li):
                st = text_of(s)
                if not CUE.search(st):
                    continue
                basis = basis_of(s)
                if not basis:
                    continue
                rows.append({'operator': op['name'], 'charId': op['charId'], 'basis': basis, 'sentence': st,
                             'section': section,
                             'alters': [name_of[c] for c in alters.get(op['charId'], []) if c in name_of], 'url': url, 'fetched': t.get('_fetched')})
                with_basis.add(op['charId'])
    with open(OUT, 'w') as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + '\n')
    n = len(operators())
    print(f'extract: {n} operators, {with_page} with a Trivia page, {len(with_basis)} with a basis '
          f'({len(rows)} sentences) -> {OUT}', file=sys.stderr)


if __name__ == '__main__':
    cmd = sys.argv[1] if len(sys.argv) > 1 else 'all'
    if cmd in ('fetch', 'all'):
        cmd_fetch()
    if cmd in ('extract', 'all'):
        cmd_extract()
