#!/usr/bin/env python3
"""Aggregate bench JSON into markdown tables: speed, revisions, RSS, WER vs the
offline Parakeet reference, and rough commit lag per word."""
import glob, json, os, re, statistics, sys
from collections import defaultdict

B = os.path.dirname(os.path.abspath(__file__))
REF_DIR = os.path.join(B, "results/offline")
RUN_DIRS = sys.argv[1:] or [os.path.join(B, "results/main"), os.path.join(B, "results/keyterms")]

def norm_words(s):
    s = s.lower().replace("’", "'")
    s = re.sub(r"[^a-z0-9' ]+", " ", s)
    return s.split()

def wer(ref, hyp):
    r, h = norm_words(ref), norm_words(hyp)
    d = list(range(len(h) + 1))
    for i in range(1, len(r) + 1):
        prev, d[0] = d[0], i
        for j in range(1, len(h) + 1):
            cur = d[j]
            d[j] = min(d[j] + 1, d[j - 1] + 1, prev + (r[i - 1] != h[j - 1]))
            prev = cur
    return d[len(h)], len(r)

refs = {}
for p in glob.glob(os.path.join(REF_DIR, "*.json")):
    j = json.load(open(p)); refs[j["file"]] = j

runs = defaultdict(list)  # key -> list of per-file json
for d in RUN_DIRS:
    for p in sorted(glob.glob(os.path.join(d, "*.json"))):
        j = json.load(open(p))
        key = (j["engine"], j["model"], j.get("threads", 0))
        runs[key].append(j)

def commit_lags(j, ref):
    """Match committed words to offline word times in order; lag = commit audio time - word time."""
    words = ref.get("words", [])
    if not words: return []
    ri = 0; out = []
    for e in j.get("commit_log", []):
        for w in norm_words(e["added"]):
            for k in range(ri, min(ri + 6, len(words))):
                if norm_words(words[k]["w"]) == [w]:
                    out.append((w, words[k]["t"], e["t"], e["t"] - words[k]["t"])); ri = k + 1; break
    return out

rows = []
lag_examples = {}
for key, js in sorted(runs.items()):
    eng, model, thr = key
    audio = sum(j["audio_s"] for j in js)
    comp = sum(j["total_compute_s"] for j in js)
    means = [j["chunk_mean_ms"] for j in js]
    p95s = [j["chunk_p95_ms"] for j in js]
    maxs = [j["chunk_max_ms"] for j in js]
    fins = [j["finalize_ms"] for j in js]
    revs = sum(len(j.get("revisions", [])) for j in js)
    rss = max(j["peak_rss_mb"] for j in js)
    load = statistics.mean(j["load_ms"] for j in js)
    errs = n = 0
    for j in js:
        if j["file"] in refs:
            e, c = wer(refs[j["file"]]["final_text"], j["final_text"]); errs += e; n += c
    lags = []
    for j in js:
        if j["file"] in refs: lags += commit_lags(j, refs[j["file"]])
    lag_med = statistics.median(l[3] for l in lags) if lags else float("nan")
    lag_examples[key] = lags
    rows.append((eng, model, thr, len(js), load, statistics.mean(means), max(p95s), max(maxs), comp / audio, statistics.mean(fins), max(fins), revs, rss, 100.0 * errs / n if n else float("nan"), lag_med, len(lags)))

print("| engine | model | thr | files | load ms | chunk mean ms | chunk p95 ms (worst file) | chunk max ms | RTF | finalize ms mean/max | committed revisions | peak RSS MB | WER% vs Parakeet | median commit lag s (n words) |")
print("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
for r in rows:
    print(f"| {r[0]} | {r[1]} | {r[2]} | {r[3]} | {r[4]:.0f} | {r[5]:.1f} | {r[6]:.0f} | {r[7]:.0f} | {r[8]:.3f} | {r[9]:.0f} / {r[10]:.0f} | {r[11]} | {r[12]:.0f} | {r[13]:.1f} | {r[14]:.2f} ({r[15]}) |")

print("\n## Per-file RTF\n")
files = sorted(refs)
print("| run | " + " | ".join(files) + " |")
print("|---|" + "---|" * len(files))
for key, js in sorted(runs.items()):
    by = {j["file"]: j for j in js}
    print(f"| {key[0]} {key[1]} t{key[2]} | " + " | ".join(f"{by[f]['rtf']:.3f}" if f in by else "-" for f in files) + " |")

print("\n## Per-file WER% vs Parakeet offline\n")
print("| run | " + " | ".join(files) + " |")
print("|---|" + "---|" * len(files))
for key, js in sorted(runs.items()):
    by = {j["file"]: j for j in js}
    cells = []
    for f in files:
        if f in by and f in refs:
            e, c = wer(refs[f]["final_text"], by[f]["final_text"]); cells.append(f"{100*e/c:.1f}")
        else: cells.append("-")
    print(f"| {key[0]} {key[1]} t{key[2]} | " + " | ".join(cells) + " |")

print("\n## Commit lag examples (word, spoken at s per Parakeet, committed at s, lag s)\n")
for key, lags in sorted(lag_examples.items()):
    if not lags: continue
    pick = lags[:: max(1, len(lags) // 6)][:6]
    print(f"- {key[0]} {key[1]} t{key[2]}: " + "; ".join(f"{w} {a:.2f}->{c:.2f} (+{l:.2f})" for w, a, c, l in pick))

print("\n## Revisions of committed text\n")
for key, js in sorted(runs.items()):
    for j in js:
        for r in j.get("revisions", []):
            print(f"- {key[0]} {key[1]} t{key[2]} {j['file']} chunk {r['chunk']} (t={r['t']:.2f}): `{r['old'][-60:]}` -> `{r['new'][-60:]}`")

print("\n## Transcripts\n")
for f in files:
    print(f"### {f} ({refs[f]['audio_s']:.1f} s)\n")
    print(f"- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: {refs[f]['final_text']}")
    for key, js in sorted(runs.items()):
        by = {j["file"]: j for j in js}
        if f in by:
            print(f"- {key[0]} {key[1]} t{key[2]}: {by[f]['final_text']}")
    print()
