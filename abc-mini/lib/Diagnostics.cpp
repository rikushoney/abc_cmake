#include "DiagnosticsImpl.h"

#include <cstdio>

using namespace abc_mini;

namespace abc_mini::detail {
void default_fault_handler(const char *Reason) {
  std::printf("AbcMini error: %s\n", Reason);
}
} // namespace abc_mini::detail

FaultHandlerStack::FaultHandlerVec &FaultHandlerStack::handlers() {
  static FaultHandlerVec handlers{detail::default_fault_handler};
  return handlers;
}

void AbcMiniInstallFaultHandler(AbcMiniFaultHandler Handler) {
  FaultHandlerStack::FaultHandlerVec &handlers = FaultHandlerStack::handlers();
  if (handlers.size() == 1 && handlers[0] == detail::default_fault_handler) {
    handlers[0] = Handler;
  } else {
    handlers.push_back(Handler);
  }
}

void AbcMiniResetFaultHandlers() {
  FaultHandlerStack::FaultHandlerVec &handlers = FaultHandlerStack::handlers();
  handlers.clear();
  handlers.push_back(detail::default_fault_handler);
}
