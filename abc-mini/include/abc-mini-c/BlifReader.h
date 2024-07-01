#ifndef _ABC_MINI_C_BLIF_READER_H_
#define _ABC_MINI_C_BLIF_READER_H_

#include "abc-mini-c/ExternC.h"
#include "abc-mini-c/Types.h"

ABC_MINI_EXTERN_C_BEGIN

AbcResult AbcMiniReadBlif(const char *Text, AbcNetwork *OutNetwork);

ABC_MINI_EXTERN_C_END

#endif
