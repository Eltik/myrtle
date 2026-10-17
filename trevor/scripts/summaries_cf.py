#!/usr/bin/env python3
"""P2 story and group summaries for NEW stories on the VPS: Cloudflare Workers AI in the hour before the daily
Neuron reset, else the local Gemma 4 E2B overnight (design/trevor-vps-runbook.md section 5; decided 2026-10-16:
summaries are the only model stage of the daily update).

The rows are p2-summaries.py's, in the same files (artifacts/p2/stories.jsonl, groups.jsonl), keyed the same way
(scripts/incr.py: unit id, sha of the exact model input, sha of prompts/summary.{story,group}.txt), with the same
system prompts, user messages and max_tokens (320 story, 700 group). Rows add `backend` ("cf" or "local") and, for
Cloudflare, `neurons`. A unit is redone here only when it is NEW (no row); `--scope stale` also redoes units whose
input or prompt changed (p2-summaries.py's rule).

Backends, in order (`--backend auto`, the default):
  cf     only inside TREVOR_CF_WINDOW (UTC, default 23:00-23:55, the hour before the 00:00 UTC reset, so it spends
         what the day's readers left), only with TREVOR_CF_ACCOUNT_ID and TREVOR_CF_API_TOKEN set, and only while
         this run's spend stays under TREVOR_CF_SUMMARY_MAX_NEURONS (default 9000). Models: TREVOR_CF_MODELS
         (comma list, default @cf/google/gemma-4-26b-a4b-it); a model the API does not know moves to the next id.
         A 429 or a daily-allocation error stops the cf backend for the run (Cloudflare's free plan hard-stops, it
         never bills). Thinking is turned off the way the worker does it (TREVOR_CF_THINKING: unset =
         chat_template_kwargs.enable_thinking false, `effort` = reasoning_effort "none", `omit` = neither; not
         verified against Cloudflare's Gemma 4 page); TREVOR_CF_EXTRA (a JSON object) is merged into each body.
         TREVOR_CF_BASE_URL (a full chat-completions URL) replaces Cloudflare's, as in the worker (tests). Neurons = ceil(in x 9091 / 1e6 + out x 27273 / 1e6)
         from the response's usage (contract v1 section 2).
  local  an OpenAI-compatible llama-server at TREVOR_SUMMARY_LLM_URL (default http://127.0.0.1:8082), started by
         vps-update.sh with the E2B GGUF and a 16,384-token context (the serving llama-server's 4,096 cannot hold a
         story: the longest P2 prompt is 12,863 tokens), until --until (UTC, default 07:00).
Exit 0 when the run ended normally (units may remain for the next night), 1 on an unexpected error. The run's
counts go to stdout as one JSON line and to --status FILE.

Usage: summaries_cf.py [--backend auto|cf|local] [--scope new|stale] [--no-groups] [--plan] [--status FILE]
       summaries_cf.py --self-test      (a local fake OpenAI server; no network)
"""
import argparse, datetime, json, math, os, sys, time, urllib.error, urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))  # common.py and incr.py sit beside the scripts
import common  # noqa: E402

CF_URL = "https://api.cloudflare.com/client/v4/accounts/{account}/ai/v1/chat/completions"  # openai.rs cloudflare_url
DEFAULT_MODELS = "@cf/google/gemma-4-26b-a4b-it"
STORY_TOKENS, GROUP_TOKENS = 320, 700


def neurons(tokens_in, tokens_out):
    return math.ceil(tokens_in * 9091 / 1e6 + tokens_out * 27273 / 1e6)


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc)


def in_window(now, window):
    a, b = window.split("-")
    t = now.strftime("%H:%M")
    return a <= t < b if a <= b else (t >= a or t < b)


def next_at(now, hhmm):
    h, m = map(int, hhmm.split(":"))
    t = now.replace(hour=h, minute=m, second=0, microsecond=0)
    return t if t > now else t + datetime.timedelta(days=1)


class Quota(Exception):
    """The cf backend is done for this run (quota, budget, window or no model left)."""


class Backend:
    name = "?"

    def chat(self, system, user, max_tokens):
        """(text, prompt tokens, generated tokens, model, neurons or None)"""
        raise NotImplementedError


class CF(Backend):
    name = "cf"

    def __init__(self, env, now_fn=utc_now, sleep=time.sleep):
        self.account, self.token = env.get("TREVOR_CF_ACCOUNT_ID"), env.get("TREVOR_CF_API_TOKEN")
        self.url = env.get("TREVOR_CF_BASE_URL") or CF_URL.format(account=self.account)
        self.models = [m.strip() for m in (env.get("TREVOR_CF_MODELS") or DEFAULT_MODELS).split(",") if m.strip()]
        # Thinking off as the worker sends it (src/llm/openai.rs, TREVOR_CF_THINKING).
        thinking = env.get("TREVOR_CF_THINKING")
        self.extra = ({"reasoning_effort": "none"} if thinking == "effort" else {} if thinking == "omit"
                      else {"chat_template_kwargs": {"enable_thinking": False}})
        self.extra.update(json.loads(env.get("TREVOR_CF_EXTRA") or "{}"))
        self.cap = int(env.get("TREVOR_CF_SUMMARY_MAX_NEURONS") or 9000)
        self.window = env.get("TREVOR_CF_WINDOW") or "23:00-23:55"
        self.base_url_set = bool(env.get("TREVOR_CF_BASE_URL"))
        self.spent, self.stopped, self.now, self.sleep = 0, None, now_fn, sleep

    def usable(self):
        if not ((self.account and self.token) or self.base_url_set):
            self.stopped = self.stopped or "TREVOR_CF_ACCOUNT_ID / TREVOR_CF_API_TOKEN not set"
        elif not in_window(self.now(), self.window):
            self.stopped = self.stopped or f"outside the window {self.window} UTC"
        elif self.spent >= self.cap:
            self.stopped = self.stopped or f"run cap {self.cap} neurons reached"
        elif not self.models:
            self.stopped = self.stopped or "no model id left"
        return self.stopped is None

    def chat(self, system, user, max_tokens):
        if not self.usable():
            raise Quota(self.stopped)
        while self.models:
            model = self.models[0]
            body = dict(common.chat_body(system, user, max_tokens), model=model, **self.extra)
            hdr = {"content-type": "application/json"}
            if self.token:
                hdr["authorization"] = f"Bearer {self.token}"
            req = lambda: urllib.request.Request(self.url, json.dumps(body).encode(), hdr)
            err = None
            for attempt in range(4):
                try:
                    with urllib.request.urlopen(req(), timeout=600) as r:
                        d = json.load(r)
                    u = d.get("usage") or {}
                    ti, to = int(u.get("prompt_tokens") or 0), int(u.get("completion_tokens") or 0)
                    n = neurons(ti, to)
                    self.spent += n
                    return common.content(d).strip(), ti, to, model, n
                except urllib.error.HTTPError as e:
                    txt = e.read()[:600].decode("utf-8", "replace")
                    low = txt.lower()
                    if e.code == 429 or "4006" in low or "allocation" in low or "neuron" in low:
                        self.stopped = f"quota: HTTP {e.code} {txt[:160]}"
                        raise Quota(self.stopped)
                    if e.code in (400, 404) and ("model" in low or "no route" in low or "not found" in low):
                        print(f"cf: model {model} unavailable ({e.code}); next id", flush=True)
                        self.models.pop(0)
                        break
                    if e.code < 500:
                        raise RuntimeError(f"cf: HTTP {e.code}: {txt[:300]}")
                    err = RuntimeError(f"cf: HTTP {e.code}")
                except (urllib.error.URLError, OSError, ValueError, KeyError, IndexError) as e:
                    err = e
                self.sleep(5 * (attempt + 1))
            else:
                raise RuntimeError(f"cf: {err}")
        self.stopped = "no model id left"
        raise Quota(self.stopped)


class Local(Backend):
    name = "local"

    def __init__(self, url, until):
        self.url, self.until, self.model, self.stopped = url.rstrip("/"), until, None, None

    def usable(self):
        if self.stopped:
            return False
        if utc_now() >= self.until:
            self.stopped = f"deadline {self.until:%H:%M} UTC"
            return False
        if self.model is None:
            try:
                self.model = common.model_id(self.url, 10)
            except (urllib.error.URLError, OSError, ValueError, KeyError, IndexError) as e:
                self.stopped = f"no llama-server at {self.url} ({e})"
                return False
        return True

    def chat(self, system, user, max_tokens):
        d = common.chat(self.url, system, user, max_tokens, 1800, lambda d: d)
        t = d.get("timings") or {}
        u = d.get("usage") or {}
        return (common.content(d).strip(), int(t.get("prompt_n") or u.get("prompt_tokens") or 0),
                int(t.get("predicted_n") or u.get("completion_tokens") or 0), self.model, None)


def run(a, env=os.environ):
    p2 = common.load_script("p2-summaries")
    backends = []
    if a.backend in ("auto", "cf"):
        backends.append(CF(env))
    if a.backend in ("auto", "local"):
        backends.append(Local(env.get("TREVOR_SUMMARY_LLM_URL") or "http://127.0.0.1:8082", next_at(utc_now(), a.until)))
    counts = {"story": {"todo": 0, "done": 0}, "group": {"todo": 0, "done": 0}, "byBackend": {}, "neurons": 0}

    def pick():
        for b in backends:
            if b.usable():
                return b
        return None

    def ask(system, user, max_tokens):
        while True:
            b = pick()
            if b is None:
                return None
            try:
                return b, b.chat(system, user, max_tokens)
            except Quota:
                continue

    # Stories.
    st, order, _, by_story, facts = p2.story_stage()
    reasons = {k: st.reason(k) for k in order}
    want = [k for k in order if reasons[k] == "new" or (a.scope == "stale" and reasons[k])]
    counts["story"]["todo"] = len(want)
    stopped = None
    if a.plan:
        print(f"story: {len(want)} to do ({a.scope}); first: {want[:8]}")
    else:
        if want:
            st.plan(order, show=True)  # drops rows of stories that no longer exist, as p2-summaries.py does
        for s in want:
            t0 = time.time()
            r = ask(p2.STORY_SYS, f"{facts[s]['header']}\n\nScript:\n{p2.story_input(by_story[s])}", STORY_TOKENS)
            if r is None:
                stopped = "; ".join(f"{b.name}: {b.stopped}" for b in backends)
                break
            b, (text, ti, to, model, n) = r
            row = {"storyId": s, "groupId": by_story[s][0]["groupId"], "header": facts[s]["header"],
                   "officialSynopsis": facts[s]["synopsis"], "summary": text, "model": model, "promptSha": st.psha,
                   "promptTokens": ti, "genTokens": to, "seconds": round(time.time() - t0, 1), "backend": b.name}
            if n is not None:
                row["neurons"] = n
                counts["neurons"] += n
            st.put(row)
            counts["story"]["done"] += 1
            counts["byBackend"][b.name] = counts["byBackend"].get(b.name, 0) + 1
    # Groups: only once every story of the group has a summary (p2-summaries.py's rule for methods B and C).
    if not a.no_groups:
        gst, gorder, _, groups, gfacts, stories, _targets = p2.group_stage()
        greasons = {k: gst.reason(k) for k in gorder}
        gwant = [k for k in gorder if greasons[k] == "new" or (a.scope == "stale" and greasons[k])]
        counts["group"]["todo"] = len(gwant)
        if a.plan:
            print(f"group: {len(gwant)} to do ({a.scope}); first: {gwant[:6]}")
        elif gwant and stopped is None:
            gst.plan(gorder, show=True)
            for g, m in gwant:
                if m in ("B", "C") and not all(s in stories for s in groups[g]):
                    continue
                t0 = time.time()
                r = ask(p2.GROUP_SYS, f"Story summaries, in order:\n{p2.group_body(g, m, groups, gfacts, stories)}",
                        GROUP_TOKENS)
                if r is None:
                    stopped = "; ".join(f"{b.name}: {b.stopped}" for b in backends)
                    break
                b, (text, ti, to, model, n) = r
                row = {"groupId": g, "method": m, "summary": text, "storyIds": groups[g], "model": model,
                       "promptSha": gst.psha, "promptTokens": ti, "seconds": round(time.time() - t0, 1),
                       "backend": b.name}
                if n is not None:
                    row["neurons"] = n
                    counts["neurons"] += n
                gst.put(row)
                counts["group"]["done"] += 1
                counts["byBackend"][b.name] = counts["byBackend"].get(b.name, 0) + 1
    counts["stopped"] = stopped
    counts["backends"] = {b.name: b.stopped for b in backends}
    counts["written"] = counts["story"]["done"] + counts["group"]["done"]
    counts["at"] = utc_now().strftime("%Y-%m-%dT%H:%M:%SZ")
    print(json.dumps(counts), flush=True)
    if a.status:
        common.write_json_atomic(a.status, counts, indent=1)
    return 0


def self_test():
    """The cf backend against a fake OpenAI-compatible server on 127.0.0.1: window, neuron count, model failover,
    quota stop, run cap; and the window arithmetic."""
    import http.server, threading
    state = {"calls": [], "mode": "ok"}

    class H(http.server.BaseHTTPRequestHandler):
        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers["content-length"])))
            state["calls"].append((self.path, self.headers.get("authorization"), body))
            if body["model"] == "@cf/gone":
                code, out = 400, {"errors": [{"message": "No such model @cf/gone"}]}
            elif state["mode"] == "quota":
                code, out = 429, {"errors": [{"code": 4006, "message": "daily free allocation of 10,000 neurons used"}]}
            else:
                code, out = 200, {"choices": [{"message": {"content": " A summary. "}, "finish_reason": "stop"}],
                                  "usage": {"prompt_tokens": 12000, "completion_tokens": 300}}
            b = json.dumps(out).encode()
            self.send_response(code); self.send_header("content-length", str(len(b))); self.end_headers()
            self.wfile.write(b)

        def log_message(self, *x):
            pass
    srv = http.server.HTTPServer(("127.0.0.1", 0), H)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{srv.server_address[1]}/acct/a1/chat/completions"
    inside = lambda: datetime.datetime(2026, 10, 16, 23, 10, tzinfo=datetime.timezone.utc)
    outside = lambda: datetime.datetime(2026, 10, 16, 19, 30, tzinfo=datetime.timezone.utc)
    env = {"TREVOR_CF_ACCOUNT_ID": "a1", "TREVOR_CF_API_TOKEN": "t1", "TREVOR_CF_BASE_URL": base,
           "TREVOR_CF_MODELS": "@cf/gone,@cf/google/gemma-4-26b-a4b-it", "TREVOR_CF_SUMMARY_MAX_NEURONS": "300"}
    cf = CF(env, now_fn=inside, sleep=lambda s: None)
    text, ti, to, model, n = cf.chat("sys", "user", 320)
    # 12,000 in and 300 out: 109.092 + 8.1819 = 117.27 -> 118 neurons (the ZC estimate for a story is about 210).
    assert (text, ti, to, model, n) == ("A summary.", 12000, 300, "@cf/google/gemma-4-26b-a4b-it", 118), (text, n)
    path, auth, body = state["calls"][-1]
    assert path == "/acct/a1/chat/completions" and auth == "Bearer t1" and body["max_tokens"] == 320
    assert body["chat_template_kwargs"] == {"enable_thinking": False} and body["temperature"] == 0
    assert body["messages"][0] == {"role": "system", "content": "sys"}
    cf.chat("sys", "user", 320)  # 236 spent
    cf.chat("sys", "user", 320)  # 354 >= cap 300: the next call refuses
    try:
        cf.chat("sys", "user", 320)
        raise AssertionError("ran past the cap")
    except Quota as e:
        assert "cap 300" in str(e)
    state["mode"] = "quota"
    cf2 = CF(dict(env, TREVOR_CF_MODELS="@cf/google/gemma-4-26b-a4b-it"), now_fn=inside, sleep=lambda s: None)
    try:
        cf2.chat("s", "u", 10)
        raise AssertionError("quota not detected")
    except Quota as e:
        assert "quota" in str(e) and not cf2.usable()
    assert CF(dict(env, TREVOR_CF_THINKING="effort")).extra == {"reasoning_effort": "none"}
    assert CF(dict(env, TREVOR_CF_THINKING="omit", TREVOR_CF_EXTRA='{"x": 1}')).extra == {"x": 1}
    assert CF({"TREVOR_CF_ACCOUNT_ID": "acc"}).url == CF_URL.format(account="acc")
    cf3 = CF(env, now_fn=outside, sleep=lambda s: None)
    assert not cf3.usable() and "window" in cf3.stopped
    assert not CF({}, now_fn=inside).usable()
    srv.shutdown()
    t = lambda h, m: datetime.datetime(2026, 10, 16, h, m, tzinfo=datetime.timezone.utc)
    assert in_window(t(23, 0), "23:00-23:55") and not in_window(t(23, 55), "23:00-23:55")
    assert in_window(t(0, 30), "23:30-01:00") and not in_window(t(1, 0), "23:30-01:00")
    assert next_at(t(23, 10), "07:00") == t(7, 0) + datetime.timedelta(days=1)
    assert neurons(1_000_000, 0) == 9091 and neurons(0, 1_000_000) == 27273
    print("summaries_cf.py self-test: ok (request shape, usage -> 118 neurons, model failover, run cap, quota stop, "
          "window, deadline)")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--backend", choices=("auto", "cf", "local"), default="auto")
    ap.add_argument("--scope", choices=("new", "stale"), default="new")
    ap.add_argument("--no-groups", action="store_true", help="story summaries only")
    ap.add_argument("--until", default="07:00", help="UTC time the local backend stops starting units")
    ap.add_argument("--plan", action="store_true", help="print what would be summarized; call nothing, write nothing")
    ap.add_argument("--status", default=None, help="also write the counts here")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        self_test()
        return 0
    try:
        return run(a)
    except (RuntimeError, OSError, ValueError, KeyError) as e:
        sys.stderr.write(f"summaries: {type(e).__name__}: {e}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
