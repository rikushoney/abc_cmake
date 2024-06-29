cmake_minimum_required(VERSION 3.20.0)

list(INSERT
  CMAKE_MODULE_PATH 0
  "${CMAKE_CURRENT_SOURCE_DIR}/cmake/modules")

string(TIMESTAMP ABC_CALVER "%Y%m%d" UTC)

project(abc
  VERSION 0.0.${ABC_CALVER}
  LANGUAGES C CXX)

if(WIN32)
  set(ABC_BUILD_DYLIB_DEFAULT 0)
  set(ABC_LINK_DYLIB_DEFAULT 0)
  set(ABC_USE_PIC_DEFAULT 0)
  set(ABC_USE_PTHREADS_DEFAULT 0)
else()
  set(ABC_BUILD_DYLIB_DEFAULT 1)
  set(ABC_LINK_DYLIB_DEFAULT 1)
  set(ABC_USE_PIC_DEFAULT 1)
  set(ABC_USE_PTHREADS_DEFAULT 1)
endif()

option(ABC_BUILD_DYLIB
  "Build ABC as a shared library"
  ${ABC_BUILD_DYLIB_DEFAULT})

option(ABC_LINK_DYLIB
  "Link the ABC executable to the shared library"
  ${ABC_BUILD_DYLIB_DEFAULT})

option(ABC_USE_PIC
  "Compile ABC using position independent code"
  ${ABC_USE_PIC_DEFAULT})

option(ABC_USE_PTHREADS
  "Compile ABC with pthreads support"
  ${ABC_USE_PTHREADS_DEFAULT})

set(ABC_C_STANDARD 11)

set_property(GLOBAL
  PROPERTY ABC_REQUIRED_C_STANDARD
  ${ABC_C_STANDARD})

set(ABC_CXX_STANDARD 17)

set_property(GLOBAL
  PROPERTY ABC_REQUIRED_CXX_STANDARD
  ${ABC_CXX_STANDARD})

set_property(GLOBAL
  APPEND
  PROPERTY ABC_INCLUDE_DIRS
  "${CMAKE_CURRENT_SOURCE_DIR}/src")

set_property(GLOBAL
  APPEND
  PROPERTY ABC_COMPILE_DEFS
  "ABC_USE_CUDD=1"
  "ABC_USE_STDINT_H=1")

set_property(GLOBAL
  APPEND
  PROPERTY ABC_COMPILE_FLAGS
  $<$<BOOL:${ABC_USE_PIC}>:-fPIC>)

set(CMAKE_EXPORT_COMPILE_COMMANDS 1)

if(${ABC_USE_PTHREADS})
  find_package(Threads)
endif()

if(${Threads_FOUND} AND ${CMAKE_USE_PTHREADS_INIT})
  set_property(GLOBAL
    APPEND
    PROPERTY ABC_COMPILE_DEFS
    "ABC_USE_PTHREADS=1")
endif()

find_package(Python3
  REQUIRED
  COMPONENTS Interpreter)

execute_process(
  COMMAND ${Python3_EXECUTABLE} parse-module-makefiles.py
  WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
  OUTPUT_VARIABLE parse_module_makefiles_stdout
  ERROR_VARIABLE parse_module_makefiles_stderr
  RESULT_VARIABLE parse_module_makefiles_result)

if(NOT ${parse_module_makefiles_result} EQUAL 0)
  message("Failed to parse module makefiles")
  message("stdout: \"${parse_module_makefiles_stdout}\"")
  message("stderr: \"${parse_module_makefiles_stderr}\"")
  if("${parse_module_makefiles_result}" MATCHES "^[0-9]+\$")
    set(error_message "exited with code ${parse_module_makefiles_result}")
  else()
    set(error_message "${parse_module_makefiles_result}")
  endif()
  message(FATAL_ERROR "parse-module-makefiles.py: ${error_message}")
endif()

file(STRINGS
  "${CMAKE_CURRENT_SOURCE_DIR}/sources.txt"
  sources_txt)

include(AddABC)

foreach(lib_entry ${sources_txt})
  string(FIND "${lib_entry}" ":" needle)
  string(SUBSTRING "${lib_entry}" 0 ${needle} libname)
  math(EXPR jump "${needle}+1")
  string(SUBSTRING "${lib_entry}" ${jump} -1 sources)

  abc_add_library(${libname} SOURCES ${sources})
endforeach()

get_property(ABC_INCLUDE_DIRS
  GLOBAL
  PROPERTY ABC_INCLUDE_DIRS)

get_property(ABC_COMPILE_DEFS
  GLOBAL
  PROPERTY ABC_COMPILE_DEFS)

get_property(ABC_COMPILE_FLAGS
  GLOBAL
  PROPERTY ABC_COMPILE_FLAGS)

get_property(ABC_STATIC_LIBS
  GLOBAL
  PROPERTY ABC_STATIC_LIBS)

if(NOT MSVC)
  set(ABC_LINK_STATIC_LIBS
    -Wl,--whole-archive
    ${ABC_STATIC_LIBS}
    -Wl,--no-whole-archive)
else()
  set(ABC_LINK_STATIC_LIBS ${ABC_STATIC_LIBS})
endif()

if(ABC_BUILD_DYLIB)
  add_library(abc
    SHARED
    "${CMAKE_CURRENT_SOURCE_DIR}/src/libabc.c")

  target_link_libraries(abc
    PRIVATE
    ${ABC_LINK_STATIC_LIBS})

  set_target_properties(abc
    PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/lib"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/lib")
endif()

add_executable(abc_exe)

target_sources(abc_exe
  PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/src/base/main/main.c")

target_include_directories(abc_exe
  PRIVATE ${ABC_INCLUDE_DIRS})

target_compile_definitions(abc_exe
  PRIVATE ${ABC_COMPILE_DEFS})

target_compile_options(abc_exe
  PRIVATE ${ABC_COMPILE_FLAGS})

if(${ABC_BUILD_DYLIB} AND ${ABC_LINK_DYLIB})
  target_link_libraries(abc_exe PRIVATE abc)
else()
  target_link_libraries(abc_exe
    PRIVATE ${ABC_LINK_STATIC_LIBS})
endif()

set_target_properties(abc_exe
  PROPERTIES
  C_STANDARD_REQUIRED 1
  C_STANDARD "${ABC_C_STANDARD}"
  OUTPUT_NAME abc)

set_target_properties(abc_exe
  PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/bin")
