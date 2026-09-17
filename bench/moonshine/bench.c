/* Streaming benchmark harness for the Moonshine v2 C API.
 *
 * bench <model_dir> <arch> <label> <out_dir> <keyterms|-> <wav>...
 *
 * Feeds 80 ms chunks, calls moonshine_transcribe_stream after every chunk
 * (flags 0, so the library's own 200 ms throttle applies, as it would in an
 * app), tracks committed (= is_complete lines) vs tentative text, and writes
 * one JSON per wav in the same schema as the Rust harnesses.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include "moonshine-c-api.h"

#define CHUNK 1280
#define MAX_LINES 4096

static double now_ms(void) {
  struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
  return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

static double peak_rss_mb(void) {
  FILE *f = fopen("/proc/self/status", "r"); char line[256]; double kb = 0;
  while (f && fgets(line, sizeof line, f)) if (sscanf(line, "VmHWM: %lf", &kb) == 1) break;
  if (f) fclose(f); return kb / 1024.0;
}

static float *read_wav(const char *path, size_t *n) {
  FILE *f = fopen(path, "rb"); if (!f) { perror(path); exit(1); }
  unsigned char hdr[12]; fread(hdr, 1, 12, f);
  /* walk chunks to "data" */
  for (;;) {
    unsigned char ch[8]; if (fread(ch, 1, 8, f) != 8) { fprintf(stderr, "no data chunk\n"); exit(1); }
    uint32_t sz = ch[4] | ch[5] << 8 | ch[6] << 16 | (uint32_t)ch[7] << 24;
    if (!memcmp(ch, "data", 4)) {
      *n = sz / 2; int16_t *buf = malloc(sz); fread(buf, 1, sz, f); fclose(f);
      float *out = malloc(*n * sizeof(float));
      for (size_t i = 0; i < *n; i++) out[i] = buf[i] / 32768.0f;
      free(buf); return out;
    }
    fseek(f, sz + (sz & 1), SEEK_CUR);
  }
}

static void json_str(FILE *o, const char *s) {
  fputc('"', o);
  for (; *s; s++) {
    unsigned char c = *s;
    if (c == '"' || c == '\\') { fputc('\\', o); fputc(c, o); }
    else if (c == '\n') fputs("\\n", o);
    else if (c < 0x20) fprintf(o, "\\u%04x", c);
    else fputc(c, o);
  }
  fputc('"', o);
}

static int cmp_d(const void *a, const void *b) { double x = *(double*)a, y = *(double*)b; return (x > y) - (x < y); }

static const char *stem(const char *p, char *buf) {
  const char *s = strrchr(p, '/'); s = s ? s + 1 : p; strcpy(buf, s);
  char *dot = strrchr(buf, '.'); if (dot) *dot = 0; return buf;
}

/* committed text tracker */
typedef struct { char *committed; FILE *log; FILE *rev; int nlog, nrev; } tracker_t;

static void observe(tracker_t *t, int chunk, double at, const char *new_text) {
  if (!strcmp(t->committed, new_text)) return;
  size_t l = strlen(t->committed);
  if (!strncmp(new_text, t->committed, l)) {
    fprintf(t->log, "%s{\"chunk\":%d,\"t\":%.3f,\"added\":", t->nlog++ ? "," : "", chunk, at);
    json_str(t->log, new_text + l); fputc('}', t->log);
  } else {
    fprintf(t->rev, "%s{\"chunk\":%d,\"t\":%.3f,\"old\":", t->nrev++ ? "," : "", chunk, at);
    json_str(t->rev, t->committed); fputs(",\"new\":", t->rev); json_str(t->rev, new_text); fputc('}', t->rev);
    fprintf(t->log, "%s{\"chunk\":%d,\"t\":%.3f,\"revised\":true,\"added\":", t->nlog++ ? "," : "", chunk, at);
    json_str(t->log, new_text); fputc('}', t->log);
  }
  free(t->committed); t->committed = strdup(new_text);
}

/* Build committed (complete lines joined by space) and tentative (incomplete lines). */
static void split_transcript(const struct transcript_t *tr, char *committed, char *tentative, size_t cap) {
  committed[0] = tentative[0] = 0;
  for (uint64_t i = 0; i < tr->line_count; i++) {
    const struct transcript_line_t *l = &tr->lines[i];
    char *dst = l->is_complete ? committed : tentative;
    if (!l->text || !l->text[0]) continue;
    if (dst[0] && strlen(dst) + 1 < cap) strcat(dst, " ");
    strncat(dst, l->text, cap - strlen(dst) - 1);
  }
}

int main(int argc, char **argv) {
  if (argc < 7) { fprintf(stderr, "usage: bench <model_dir> <arch> <label> <out_dir> <keyterms|-> <wav>...\n"); return 2; }
  const char *model_dir = argv[1]; int arch = atoi(argv[2]); const char *label = argv[3];
  const char *out_dir = argv[4]; const char *keyterms = argv[5];
  struct moonshine_option_t opts[16]; uint64_t nopts = 0;
  if (strcmp(keyterms, "-")) { opts[nopts].name = "keyterms"; opts[nopts].value = keyterms; nopts++; }
  /* extra options via MOON_OPTS="name=value;name=value" */
  char *extra = getenv("MOON_OPTS") ? strdup(getenv("MOON_OPTS")) : NULL;
  for (char *tok = extra ? strtok(extra, ";") : NULL; tok && nopts < 16; tok = strtok(NULL, ";")) {
    char *eq = strchr(tok, '='); if (!eq) continue; *eq = 0; opts[nopts].name = tok; opts[nopts].value = eq + 1; nopts++;
    fprintf(stderr, "option %s=%s\n", tok, eq + 1);
  }

  double t0 = now_ms();
  int32_t h = moonshine_load_transcriber_from_files(model_dir, arch, opts, nopts, MOONSHINE_HEADER_VERSION);
  if (h < 0) { fprintf(stderr, "load failed: %s\n", moonshine_error_to_string(h)); return 1; }
  double load_ms = now_ms() - t0;
  fprintf(stderr, "loaded %s (arch %d, lib version %d) in %.0f ms, keyterms=%s\n", label, arch, moonshine_get_version(), load_ms, keyterms);

  for (int a = 6; a < argc; a++) {
    size_t n; float *pcm = read_wav(argv[a], &n); double audio_s = n / 16000.0;
    char st[256]; stem(argv[a], st);
    int32_t s = moonshine_create_stream(h, 0);
    if (s < 0) { fprintf(stderr, "create_stream: %s\n", moonshine_error_to_string(s)); return 1; }
    int32_t err = moonshine_start_stream(h, s);
    if (err) { fprintf(stderr, "start_stream: %s\n", moonshine_error_to_string(err)); return 1; }

    tracker_t tr = { strdup(""), tmpfile(), tmpfile(), 0, 0 };
    FILE *tent = tmpfile(); int ntent = 0;
    size_t nchunks = (n + CHUNK - 1) / CHUNK; double *ms = malloc(nchunks * sizeof(double));
    static char committed[1 << 16], tentative[1 << 16], last_tent[1 << 16] = "";
    struct transcript_t *out = NULL;
    for (size_t i = 0; i < nchunks; i++) {
      size_t len = (i + 1) * CHUNK <= n ? CHUNK : n - i * CHUNK;
      double t = now_ms();
      err = moonshine_transcribe_add_audio_to_stream(h, s, pcm + i * CHUNK, len, 16000, 0);
      if (err) { fprintf(stderr, "add_audio: %s\n", moonshine_error_to_string(err)); return 1; }
      err = moonshine_transcribe_stream(h, s, 0, &out);
      if (err) { fprintf(stderr, "transcribe_stream: %s\n", moonshine_error_to_string(err)); return 1; }
      ms[i] = now_ms() - t;
      double at = (i + 1) * (double)CHUNK / 16000.0;
      split_transcript(out, committed, tentative, sizeof committed);
      observe(&tr, (int)i, at, committed);
      if (strcmp(tentative, last_tent)) {
        if (ntent < 400) { fprintf(tent, "%s{\"chunk\":%zu,\"t\":%.3f,\"tentative\":", ntent ? "," : "", i, at); json_str(tent, tentative); fputc('}', tent); }
        ntent++; strcpy(last_tent, tentative);
      }
    }
    double t = now_ms();
    err = moonshine_stop_stream(h, s);
    err |= moonshine_transcribe_stream(h, s, 0, &out);
    double finalize_ms = now_ms() - t;
    if (err) { fprintf(stderr, "finalize: %s\n", moonshine_error_to_string(err)); return 1; }
    split_transcript(out, committed, tentative, sizeof committed);
    static char full[1 << 16]; full[0] = 0;
    for (uint64_t i = 0; i < out->line_count; i++) { if (!out->lines[i].text || !out->lines[i].text[0]) continue; if (full[0]) strcat(full, " "); strcat(full, out->lines[i].text); }
    observe(&tr, (int)nchunks, audio_s, committed);

    double *sorted = malloc(nchunks * sizeof(double)); memcpy(sorted, ms, nchunks * sizeof(double));
    qsort(sorted, nchunks, sizeof(double), cmp_d);
    double total = 0; for (size_t i = 0; i < nchunks; i++) total += ms[i];
    size_t p95i = (size_t)(nchunks * 0.95); if (p95i >= nchunks) p95i = nchunks - 1;

    char path[1024]; snprintf(path, sizeof path, "%s/%s_%s.json", out_dir, label, st);
    FILE *o = fopen(path, "w");
    fprintf(o, "{\"engine\":\"moonshine\",\"model\":"); json_str(o, label);
    fprintf(o, ",\"threads\":0,\"keyterms\":"); json_str(o, keyterms);
    fprintf(o, ",\"file\":"); json_str(o, st);
    fprintf(o, ",\"audio_s\":%.4f,\"load_ms\":%.1f,\"n_chunks\":%zu,\"chunk_mean_ms\":%.4f,\"chunk_p95_ms\":%.4f,\"chunk_max_ms\":%.4f,\"total_compute_s\":%.4f,\"rtf\":%.4f,\"finalize_ms\":%.2f,\"n_tentative_changes\":%d,\"n_lines\":%llu,\"peak_rss_mb\":%.1f",
            audio_s, load_ms, nchunks, total / nchunks, sorted[p95i], sorted[nchunks - 1], total / 1000.0, total / 1000.0 / audio_s, finalize_ms, ntent, (unsigned long long)out->line_count, peak_rss_mb());
    fprintf(o, ",\"final_text\":"); json_str(o, full);
    fprintf(o, ",\"final_committed\":"); json_str(o, committed);
    #define DUMP(name, f) { fprintf(o, ",\"" name "\":["); rewind(f); int c; while ((c = fgetc(f)) != EOF) fputc(c, o); fputs("]", o); }
    DUMP("commit_log", tr.log); DUMP("revisions", tr.rev); DUMP("tentative_log", tent);
    fprintf(o, ",\"chunk_ms\":["); for (size_t i = 0; i < nchunks; i++) fprintf(o, "%s%.1f", i ? "," : "", ms[i]); fputs("]", o);
    fprintf(o, "}\n"); fclose(o);
    fprintf(stderr, "%s: rtf %.3f p95 %.1f ms max %.1f ms fin %.0f ms | %s\n", st, total / 1000.0 / audio_s, sorted[p95i], sorted[nchunks - 1], finalize_ms, full);
    moonshine_free_stream(h, s); free(pcm); free(ms); free(sorted); fclose(tr.log); fclose(tr.rev); fclose(tent); free(tr.committed); last_tent[0] = 0;
  }
  moonshine_free_transcriber(h);
  return 0;
}
