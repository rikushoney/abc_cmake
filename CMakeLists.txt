cmake_minimum_required(VERSION 3.10)

project(abc)

list(APPEND CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake/modules/)

add_library(abc STATIC)
target_include_directories(abc PUBLIC ${CMAKE_SOURCE_DIR}/src)
target_compile_definitions(abc PUBLIC ABC_USE_STDINT_H=1)

find_package(Readline)
if (Readline_FOUND)
  target_link_libraries(abc PUBLIC Readline::Readline)
  target_compile_definitions(abc PUBLIC ABC_USE_READLINE)
endif()

find_package(Threads)
if (THREADS_FOUND)
  target_link_libraries(abc PUBLIC Threads::Threads)
  target_compile_definitions(abc PUBLIC ABC_USE_PTHREADS)
endif()

if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
  target_compile_options(abc PRIVATE -Wno-write-strings)
elseif (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
  target_compile_options(abc
    PRIVATE
    -Wno-implicit-const-int-float-conversion
    -Wno-parentheses-equality
    -Wno-string-plus-char
    -Wno-writable-strings
  )
endif()

add_subdirectory(src)

add_executable(abc_exe)
set_property(TARGET abc_exe PROPERTY OUTPUT_NAME abc)
target_sources(abc_exe PRIVATE ${CMAKE_SOURCE_DIR}/src/base/main/main.c)
target_link_libraries(abc_exe PRIVATE abc)
