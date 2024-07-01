#ifndef _ABC_MINI_ASSERT_H_
#define _ABC_MINI_ASSERT_H_

#ifdef ABC_MINI_ENABLE_ASSERTIONS
#ifdef NDEBUG
#define _ABC_OLD_NDEBUG NDEBUG
#undef NDEBUG
#include <assert.h>
#define NDEBUG _ABC_OLD_NDEBUG
#else
#include <assert.h>
#endif
#endif

#endif
