// ABC_MINI: Based-on: base/io/ioReadBlifMv.c,d785775f

#include "abc-mini-c/BlifReader.h"

#include "Assert.h"
#include "Log.h"

#include <memory>

using namespace abc_mini;

struct AbcNetworkImpl;
struct AbcDesignImpl;
struct BlifModule;
struct BlifManagerImpl;
struct HopManager;
struct StringTable;

// ABC_MINI: Alias-of: Vec_Ptr_t
struct Vec {
  int capacity;
  int size;
  void **array;
};

extern Vec *vGlobalLtlArray;

inline Vec *vec_alloc(int capacity) {
  auto vec = reinterpret_cast<Vec *>(malloc(sizeof(Vec)));
  if (capacity > 0 && capacity < 8) {
    capacity = 8;
  }
  vec->size = 0;
  vec->array =
      capacity ? reinterpret_cast<void **>(malloc(sizeof(void *) * capacity))
               : nullptr;
  vec->capacity = capacity;
  return vec;
}

inline void vec_grow(Vec &vec, int min_capacity) {
  if (vec.capacity >= min_capacity) {
    return;
  }
  vec.array = reinterpret_cast<void **>(
      realloc(vec.array, sizeof(void *) * min_capacity));
  vec.capacity = min_capacity;
}

inline void vec_push(Vec &vec, void *entry) {
  if (vec.size == vec.capacity) {
    if (vec.capacity < 16) {
      vec_grow(vec, 16);
    } else {
      vec_grow(vec, vec.capacity * 2);
    }
  }
  auto next_i = vec.size;
  vec.array[next_i] = entry;
  ++vec.size;
}

template <typename T> T vec_entry_at(Vec &vec, unsigned index) {
  assert(index < vec.size);
  return reinterpret_cast<T>(vec.array + index);
}

inline void vec_remove(Vec &vec, void *entry) {
  unsigned i = 0;
  for (i = vec.size - 1; i >= 0; --i) {
    if (vec.array[i] == entry) {
      break;
    }
  }
  assert(i >= 0);
  for (++i; i < vec.size; ++i) {
    vec.array[i - 1] = vec.array[i];
  }
  --vec.size;
}

extern "C" {
// ABC_MINI: Defined-in-start: base/abc/abcLib.c
AbcDesignImpl *Abc_DesCreate(char *name);
int Abc_DesFindTopLevelModels(AbcDesignImpl *design);
void Abc_DesFree(AbcDesignImpl *design, AbcNetworkImpl *network_to_keep);
// ABC_MINI: Defined-in-end

// ABC_MINI: Defined-in-start: base/abc/abcCheck.c
int Abc_NtkCheckRead(AbcNetworkImpl *network);
int Abc_NtkIsAcyclicHierarchy(AbcNetworkImpl *network);
// ABC_MINI: Defined-in-end

// ABC_MINI: Defined-in: misc/extra/extraUtilUtil.c
char *Extra_UtilStrsav(const char *s);

// ABC_MINI: Defined-in: aig/hop/hopMan.c
void Hop_ManStop(HopManager *manager);

// ABC_MINI: Defined-in-start: base/io/ioReadBlifMv.c
BlifManagerImpl *AbcMini__Io_MvAlloc();
void AbcMini__Io_MvFree(BlifManagerImpl *manager);
BlifModule *AbcMini__Io_MvModAlloc();
void AbcMini__Io_MvModFree(BlifModule *module);
void AbcMini__Io_MvReadPreparse(BlifManagerImpl *manager);
int AbcMini__Io_MvReadInterfaces(BlifManagerImpl *manager);
AbcDesignImpl *AbcMini__Io_MvParse(BlifManagerImpl *manager);
// ABC_MINI: Defined-in-end
}

// ABC_MINI: Alias-of: Abc_NtkType_t
typedef enum {
  ABC_NTK_NONE = 0,
  ABC_NTK_NETLIST,
  ABC_NTK_LOGIC,
  ABC_NTK_STRASH,
  ABC_NTK_OTHER,
} AbcMiniNetworkType;

// ABC_MINI: Alias-of: Abc_NtkFunc_t
typedef enum {
  ABC_FUNC_NONE = 0,
  ABC_FUNC_SOP,
  ABC_FUNC_BDD,
  ABC_FUNC_AIG,
  ABC_FUNC_MAP,
  ABC_FUNC_BLIFMV,
  ABC_FUNC_BLACKBOX,
  ABC_FUNC_OTHER
} AbcMiniNetworkFunc;

// ABC_MINI: Alias-of: Nm_Man_t
struct NameManager;

// ABC_MINI: Alias-of: Vec_Int_t
struct IntVec {
  int capacity;
  int size;
  int *array;
};

// ABC_MINI: Alias-of: Mem_Flex_t
struct FixedMemoryManager;

// ABC_MINI: Alias-of: Mem_Step_t
struct StepMemoryManager;

// ABC_MINI: Alias-of: Abc_ManTime_t
struct AbcTimingManager;

// ABC_MINI: Alias-of: Abc_Cex_t
struct AbcCountex;

// ABC_MINI: Alias-of: Abc_Obj_Type_t
typedef enum {
  ABC_OBJ_NONE = 0,
  ABC_OBJ_CONST1,
  ABC_OBJ_PI,
  ABC_OBJ_PO,
  ABC_OBJ_BI,
  ABC_OBJ_BO,
  ABC_OBJ_NET,
  ABC_OBJ_NODE,
  ABC_OBJ_LATCH,
  ABC_OBJ_WHITEBOX,
  ABC_OBJ_BLACKBOX,
  ABC_OBJ_NUMBER
} AbcMiniObjectType;

// ABC_MINI: Alias-of: Abc_Ntk_t
struct AbcNetworkImpl {
  AbcMiniNetworkType network_type;
  AbcMiniNetworkFunc network_functionality;
  char *name;
  char *_unused_spec;
  NameManager *name_manager;
  Vec *_unused_objects;
  Vec *_unused_primary_inputs;
  Vec *_unused_primary_outputs;
  Vec *_unused_combinational_inputs;
  Vec *_unused_combinational_outputs;
  Vec *_unused_pios;
  Vec *_unused_boxes;
  Vec *_unused_ltl_properties;
  int object_counts[ABC_OBJ_NUMBER];
  int n_live_objects;
  int _unused_n_constraints;
  int _unused_n_barrier_buffers;
  int _unused_n_barrier_buffers2;
  AbcNetworkImpl *backup;
  int _unused_step;
  AbcDesignImpl *design;
  AbcNetworkImpl *_unused_alt_view;
  int _unused_hie_visited;
  int _usused_hie_path;
  int model_id;
  double _unused_temporary_value;
  int _unused_n_traversal_ids;
  IntVec _unused_traversal_ids;
  FixedMemoryManager *_unused_object_memory_manager;
  StepMemoryManager *_unused_array_memory_manager;
  void *_unused_functionality_manager;
  AbcTimingManager *_unused_timing_manager;
  void *_unused_cut_manager;
  float _unused_and_gate_delay;
  int _unused_n_max_levels;
  IntVec *_unused_levels_reversed;
  Vec *_unused_co_support_info;
  int *_unused_counter_example_model;
  AbcCountex *_unused_sequencial_counter_example_model;
  Vec *_unused_sequencial_counter_example_model_vec;
  AbcNetworkImpl *exdc_network;
  void *_unused_exdc_care_network;
  void *_unused_misc;
  AbcNetworkImpl *_unused_copy;
  void *_unused_application_manager;
  void *_unused_sc_library;
  IntVec *_unused_sc_library_gates;
  IntVec *_unused_fanin_phases;
  char *_unused_wire_load_model;
  float *_unused_lut_times;
  Vec *_unused_onehots;
  Vec *_unused_object_permutations;
  Vec *_unused_topology;
  Vec *_unused_attribute_managers;
  Vec *_unused_name_ids;
  Vec *_unused_object_info;
  Vec *_unused_original_node_ids;
};

// ABC_MINI: Alias-of: Abc_Des_t
struct AbcDesignImpl {
  char *name;
  void *_unused_func_man;
  Vec *top_level_modules;
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
  AbcNetworkImpl *network;
  AbcObject reset_latch;
  BlifManagerImpl *manager;
};

AbcResult AbcMiniReadBlif(const char *Text, AbcNetwork *OutNetwork) {
  char filename[] = "input.blif";
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
  AbcMini__Io_MvReadPreparse(manager.inner.get());
  if (AbcMini__Io_MvReadInterfaces(manager.inner.get())) {
    design = AbcMini__Io_MvParse(manager.inner.get());
  }
  if (manager.inner->error[0]) {
    emit_fault(&manager.inner->error[0]);
  }
  if (design == nullptr) {
    return ABC_RESULT_ERROR;
  }
  for (unsigned i = 0; i < design->modules->size; ++i) {
    auto network = vec_entry_at<AbcNetworkImpl *>(*design->modules, i);
    if (!Abc_NtkCheckRead(network)) {
      emit_fault("network check has failed for {}", network->name);
      Abc_DesFree(design, nullptr);
    }
  }
  if (design->modules->size > 1) {
    auto network = vec_entry_at<AbcNetworkImpl *>(*design->modules, 0);
    for (unsigned i = 1; i < design->modules->size; ++i) {
      auto other_network = vec_entry_at<AbcNetworkImpl *>(*design->modules, i);
      if (other_network->name == std::string_view{"EXDC"}) {
        assert(network->exdc_network == nullptr);
        network->exdc_network = other_network;
        vec_remove(*design->modules, other_network);
        other_network->design = nullptr;
        --i;
      } else {
        network = other_network;
      }
    }
  }
  auto n_root_modules = Abc_DesFindTopLevelModels(design);
  auto network = vec_entry_at<AbcNetworkImpl *>(*design->top_level_modules, 0);
  if (n_root_modules > 1) {
    emit_fault("warning: the design has {} root-level modules -- the first one "
               "({}) will be used",
               design->top_level_modules->size, network->name);
  }
  network->design = design;
  design->_unused_func_man = nullptr;
  assert(design->modules->size > 0);
  if (design->modules->size == 1) {
    Abc_DesFree(design, network);
    network->design = nullptr;
    network->_unused_spec = Extra_UtilStrsav(filename);
  } else {
    if (!Abc_NtkIsAcyclicHierarchy(network)) {
      emit_fault("network ({}) hierarchy is not acyclic", network->name);
      return ABC_RESULT_ERROR;
    }
  }
  if (network->_unused_spec == nullptr) {
    network->_unused_spec = Extra_UtilStrsav(filename);
  }
  vGlobalLtlArray = vec_alloc(100);
  for (unsigned i = 0; i < vGlobalLtlArray->size; ++i) {
    auto ltl_property = vec_entry_at<char *>(*vGlobalLtlArray, i);
    vec_push(*network->_unused_ltl_properties, ltl_property);
  }
  free(vGlobalLtlArray->array);
  free(vGlobalLtlArray);
  *OutNetwork = reinterpret_cast<AbcNetwork>(network);
  return ABC_RESULT_OK;
}
