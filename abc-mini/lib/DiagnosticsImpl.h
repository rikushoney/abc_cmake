#ifndef _ABC_MINI_DIAGNOSTICS_IMPL_H_
#define _ABC_MINI_DIAGNOSTICS_IMPL_H_

#include "abc-mini-c/Diagnostics.h"

#include <vector>

namespace abc_mini {

struct FaultHandlerStack {
  using FaultHandlerVec = std::vector<AbcMiniFaultHandler>;

  static FaultHandlerVec &handlers();
};

} // namespace abc_mini

#endif
