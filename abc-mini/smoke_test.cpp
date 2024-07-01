#include "abc-mini-c/BlifReader.h"
#include "abc-mini-c/Diagnostics.h"

#include <cstdio>

void fault_log(const char *Reason) {
  std::printf("ABC MINI ERROR:\n");
  std::printf("%s\n", Reason);
}

int main() {
  AbcMiniInstallFaultHandler(fault_log);
  AbcNetwork network = nullptr;
  auto result = AbcMiniReadBlif(nullptr, &network);
  if (result == ABC_RESULT_ERROR) {
    return 1;
  }
  return 0;
}
