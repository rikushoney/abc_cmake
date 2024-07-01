add_library(abc-mini-static STATIC)

set(ABC_MINI_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../include")

target_include_directories(
  abc-mini-static
  PUBLIC
  "${ABC_MINI_INCLUDE_DIR}"
)

set(assertions_enabled $<BOOL:${ABC_MINI_ENABLE_ASSERTIONS}>)
set(debug_info_enabled $<CONFIG:Debug,RelWithDebInfo>)

target_compile_definitions(
  abc-mini-static
  PRIVATE
  $<${assertions_enabled}:ABC_MINI_ENABLE_ASSERTIONS=1>
  $<${debug_info_enabled}:ABC_MINI_EMIT_DEBUG_INFO=1>
)

set_target_properties(
  abc-mini-static
  PROPERTIES
  CXX_STANDARD_REQUIRED 1
  CXX_STANDARD 23
)

list(
  APPEND
  sources
  BlifReader.cpp
  Diagnostics.cpp
)

list(
  APPEND
  headers
  Assert.h
  DiagnosticsImpl.h
  Log.h
)

target_sources(
  abc-mini-static
  PRIVATE
  ${sources}
  ${headers}
  $<TARGET_OBJECTS:ABCC>
  $<TARGET_OBJECTS:ABCCXX>
)
