#include <stdio.h>
#include <stdlib.h>
#include "moonshine-c-api.h"
int main(int argc, char **argv) {
  struct moonshine_option_t opt = {"model_arch", argv[1]};
  char *json = NULL;
  int32_t err = moonshine_get_stt_dependencies("en", &opt, 1, &json);
  if (err) { printf("err %d\n", err); return 1; }
  printf("%s\n", json); free(json); return 0;
}
