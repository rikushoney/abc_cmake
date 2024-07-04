// ABC_MINI: Based-on: base/io/ioReadBlifMv.c,d785775f

#include "abc-mini-c/BlifReader.h"

#include "Assert.h"
#include "Log.h"

#include <memory>

using namespace abc_mini;

struct AbcDesignImpl;
struct BlifModule;
struct BlifManagerImpl;
struct HopManager;
struct StringTable;

// ABC_MINI: Alias-of: Vec_Ptr_t
struct Vec;

extern "C" {
// ABC_MINI: Defined-in: base/abc/abcLib.c
AbcDesign Abc_DesCreate(char *name);
void Hop_ManStop(HopManager *manager);

// ABC_MINI: Defined-in-start: base/io/ioReadBlifMv.c
BlifManagerImpl *AbcMini__Io_MvAlloc();
void AbcMini__Io_MvFree(BlifManagerImpl *manager);
BlifModule *AbcMini__Io_MvModAlloc();
void AbcMini__Io_MvModFree(BlifModule *module);
// ABC_MINI: Defined-in-end
}

struct AbcDesignImpl {
  char *name;
  void *_unused_func_man;
  Vec *_unused_top_level_modules;
  Vec *modules;
  StringTable *_unused_module_table;
  AbcDesignImpl *_unused_library;
  void *_unused_genlib;
};

struct BlifManagerDeleter {
  void operator()(BlifManagerImpl *manager) { AbcMini__Io_MvFree(manager); }
};

struct BlifManager {
  using BlifManagerWrapper =
      std::unique_ptr<BlifManagerImpl, BlifManagerDeleter>;

  BlifManagerWrapper inner;

  BlifManager() { inner = BlifManagerWrapper(AbcMini__Io_MvAlloc()); }
};

constexpr unsigned BLIF_MANAGER_ERROR_MAX = 512;

// ABC_MINI: Alias-of: Io_MvMan_t
struct BlifManagerImpl {
  int _unused_is_blifmv;
  int _unused_use_reset;
  char *filename;
  char *buffer;
  Vec *lines;
  AbcDesignImpl *design;
  int _unused_n_ndnodes;
  Vec *models;
  BlifModule *module;
  Vec *_unused_tokens;
  Vec *_unused_tokens2;
  Vec *_unused_func;
  char error[BLIF_MANAGER_ERROR_MAX];
  int _unused_tables_read;
  int _unused_tables_left;
};

// ABC_MINI: Alias-of: Io_MvMod_t
struct BlifModule {
  char *name;
  Vec *inputs;
  Vec *outputs;
  Vec *latches;
  Vec *_unused_flops;
  Vec *resets;
  Vec *names;
  Vec *subckt;
  Vec *_unused_shorts;
  Vec *_unused_onehots;
  Vec *_unused_mvs;
  Vec *_unused_contraints;
  Vec *_unused_ltlproperties;
  int _unused_is_black_box;
  AbcNetwork network;
  AbcObject reset_latch;
  BlifManagerImpl *manager;
};

AbcResult AbcMiniReadBlif(const char *Text, AbcNetwork *OutNetwork) {
  assert(Text != nullptr);
  assert(OutNetwork == nullptr);
  *OutNetwork = nullptr;
  auto manager = BlifManager();
  manager.inner->_unused_is_blifmv = 0;
  manager.inner->_unused_use_reset = 0;
  manager.inner->filename = nullptr;
  manager.inner->buffer = nullptr;
  std::string design_name = "";
  auto design = Abc_DesCreate(const_cast<char *>(design_name.c_str()));
  auto func_man =
      reinterpret_cast<HopManager *>(manager.inner->design->_unused_func_man);
  Hop_ManStop(func_man);
  manager.inner->design->_unused_func_man = nullptr;

  // TODO(rikus):
#if 0
  // prepare the file for parsing
  Io_MvReadPreparse(p);
  // parse interfaces of each network and construct the network
  if (Io_MvReadInterfaces(p))
    pDesign = Io_MvParse(p);
  if (p->sError[0])
    fprintf(stdout, "%s\n", p->sError);
  Io_MvFree(p);
  if (pDesign == NULL)
    return NULL;
  // pDesign should be linked to all models of the design

  // make sure that everything is okay with the network structure
  if (fCheck) {
    Vec_PtrForEachEntry(Abc_Ntk_t *, pDesign->vModules, pNtk, i) {
      if (!Abc_NtkCheckRead(pNtk)) {
        printf("Io_ReadBlifMv: The network check has failed for model %s.\n",
               pNtk->pName);
        Abc_DesFree(pDesign, NULL);
        return NULL;
      }
    }
  }

  // Abc_DesPrint( pDesign );

  // check if there is an EXDC network
  if (Vec_PtrSize(pDesign->vModules) > 1) {
    pNtk = (Abc_Ntk_t *)Vec_PtrEntry(pDesign->vModules, 0);
    Vec_PtrForEachEntryStart(Abc_Ntk_t *, pDesign->vModules, pExdc, i,
                             1) if (!strcmp(pExdc->pName, "EXDC")) {
      assert(pNtk->pExdc == NULL);
      pNtk->pExdc = pExdc;
      Vec_PtrRemove(pDesign->vModules, pExdc);
      pExdc->pDesign = NULL;
      i--;
    }
    else pNtk = pExdc;
  }

  // detect top-level model
  RetValue = Abc_DesFindTopLevelModels(pDesign);
  pNtk = (Abc_Ntk_t *)Vec_PtrEntry(pDesign->vTops, 0);
  if (RetValue > 1)
    printf("Warning: The design has %d root-level modules. The first one (%s) "
           "will be used.\n",
           Vec_PtrSize(pDesign->vTops), pNtk->pName);

  // extract the master network
  pNtk->pDesign = pDesign;
  pDesign->pManFunc = NULL;

  // verify the design for cyclic dependence
  assert(Vec_PtrSize(pDesign->vModules) > 0);
  if (Vec_PtrSize(pDesign->vModules) == 1) {
    //        printf( "Warning: The design is not hierarchical.\n" );
    Abc_DesFree(pDesign, pNtk);
    pNtk->pDesign = NULL;
    pNtk->pSpec = Extra_UtilStrsav(pFileName);
  } else
    Abc_NtkIsAcyclicHierarchy(pNtk);

  // Io_WriteBlifMv( pNtk, "_temp_.mv" );
  if (pNtk->pSpec == NULL)
    pNtk->pSpec = Extra_UtilStrsav(pFileName);

  vGlobalLtlArray = Vec_PtrAlloc(100);
  Vec_PtrForEachEntry(char *, vGlobalLtlArray, pLtlProp, i)
      Vec_PtrPush(pNtk->vLtlProperties, pLtlProp);
  Vec_PtrFreeP(&vGlobalLtlArray);
#endif

  return ABC_RESULT_ERROR;
}
