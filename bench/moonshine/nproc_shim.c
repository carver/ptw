/* LD_PRELOAD shim: redirect CPU-topology reads so onnxruntime/cpuinfo sees FAKE_NPROC CPUs. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <fcntl.h>
#include <sched.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
static int fake(void) { const char *e = getenv("FAKE_NPROC"); return e ? atoi(e) : 0; }
static const char *redirect(const char *path, char *buf) {
  const char *root = getenv("FAKE_SYS"); if (!root || !path) return path;
  if (!strncmp(path, "/sys/devices/system/cpu", 23)) { snprintf(buf, 4096, "%s%s", root, path); return buf; }
  if (!strcmp(path, "/proc/cpuinfo")) { snprintf(buf, 4096, "%s/cpuinfo", root); return buf; }
  return path;
}
long sysconf(int name) {
  static long (*real)(int); if (!real) real = dlsym(RTLD_NEXT, "sysconf");
  int n = fake(); if (n && (name == _SC_NPROCESSORS_ONLN || name == _SC_NPROCESSORS_CONF)) return n;
  return real(name);
}
int get_nprocs(void) { int n = fake(); return n ? n : (int)sysconf(_SC_NPROCESSORS_ONLN); }
int get_nprocs_conf(void) { return get_nprocs(); }
int sched_getaffinity(pid_t pid, size_t sz, cpu_set_t *set) {
  static int (*real)(pid_t, size_t, cpu_set_t *); if (!real) real = dlsym(RTLD_NEXT, "sched_getaffinity");
  int r = real(pid, sz, set); int n = fake();
  if (n && r == 0) { CPU_ZERO_S(sz, set); for (int i = 0; i < n; i++) CPU_SET_S(i, sz, set); }
  return r;
}
int open(const char *p, int flags, ...) {
  static int (*real)(const char *, int, ...); if (!real) real = dlsym(RTLD_NEXT, "open");
  char b[4096]; mode_t m = 0; if (flags & O_CREAT) { va_list ap; va_start(ap, flags); m = va_arg(ap, mode_t); va_end(ap); }
  return real(redirect(p, b), flags, m);
}
int open64(const char *p, int flags, ...) {
  static int (*real)(const char *, int, ...); if (!real) real = dlsym(RTLD_NEXT, "open64");
  char b[4096]; mode_t m = 0; if (flags & O_CREAT) { va_list ap; va_start(ap, flags); m = va_arg(ap, mode_t); va_end(ap); }
  return real(redirect(p, b), flags, m);
}
int openat(int fd, const char *p, int flags, ...) {
  static int (*real)(int, const char *, int, ...); if (!real) real = dlsym(RTLD_NEXT, "openat");
  char b[4096]; mode_t m = 0; if (flags & O_CREAT) { va_list ap; va_start(ap, flags); m = va_arg(ap, mode_t); va_end(ap); }
  return real(fd, redirect(p, b), flags, m);
}
FILE *fopen(const char *p, const char *mode) {
  static FILE *(*real)(const char *, const char *); if (!real) real = dlsym(RTLD_NEXT, "fopen");
  char b[4096]; return real(redirect(p, b), mode);
}
FILE *fopen64(const char *p, const char *mode) {
  static FILE *(*real)(const char *, const char *); if (!real) real = dlsym(RTLD_NEXT, "fopen64");
  char b[4096]; return real(redirect(p, b), mode);
}
