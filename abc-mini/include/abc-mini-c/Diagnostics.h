#ifndef _ABC_MINI_C_DIAGNOSTICS_H_
#define _ABC_MINI_C_DIAGNOSTICS_H_

#include "abc-mini-c/ExternC.h"
#include "abc-mini-c/Types.h"

ABC_MINI_EXTERN_C_BEGIN

typedef void (*AbcMiniFaultHandler)(const char *Reason);

void AbcMiniInstallFaultHandler(AbcMiniFaultHandler Handler);

void AbcMiniResetFaultHandlers();

ABC_MINI_EXTERN_C_END

#endif
