#ifndef _ABC_MINI_C_TYPES_H_
#define _ABC_MINI_C_TYPES_H_

#include "abc-mini-c/ExternC.h"

ABC_MINI_EXTERN_C_BEGIN

typedef enum AbcResult {
  ABC_RESULT_OK = 0,
  ABC_RESULT_ERROR = 1,
} AbcResult;

typedef struct Abc_Ntk_t *AbcNetwork;

typedef struct Abc_Obj_t *AbcObject;

typedef struct Abc_Des_t *AbcDesign;

typedef struct Abc_Aig_t *AbcAig;

ABC_MINI_EXTERN_C_END

#endif
