cmake_minimum_required(VERSION 3.10)

if (WIN32)
  set(
    CMAKE_TOOLCHAIN_FILE
    "${CMAKE_CURRENT_SOURCE_DIR}/vcpkg/scripts/buildsystems/vcpkg.cmake"
    CACHE
    STRING
    "Vcpkg toolchain file"
  )
endif()

project(abc)

list(APPEND CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake/modules/)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

option(ABC_NAMESPACE "Compile abc in namespace" OFF)

add_library(abc STATIC)
target_include_directories(abc PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/src)
target_compile_definitions(abc PUBLIC ABC_USE_STDINT_H=1)

if (WIN32)
  find_package(unofficial-readline-win32 CONFIG REQUIRED)
  target_link_libraries(abc PUBLIC unofficial::readline-win32::readline)
  target_compile_definitions(abc PUBLIC ABC_USE_READLINE)

  find_package(PThreads4W REQUIRED)
  target_link_libraries(abc PUBLIC PThreads4W::PThreads4W)
  target_compile_definitions(abc PUBLIC ABC_USE_PTHREADS)

  target_compile_definitions(abc PUBLIC WIN32_NO_DLL)
else()
  find_package(Readline REQUIRED)
  target_link_libraries(abc PUBLIC Readline::Readline)
  target_compile_definitions(abc PUBLIC ABC_USE_READLINE)

  find_package(Threads REQUIRED)
  target_link_libraries(abc PUBLIC Threads::Threads)
  target_compile_definitions(abc PUBLIC ABC_USE_PTHREADS)
endif()

find_package(ZLIB REQUIRED)
target_link_libraries(abc PUBLIC ZLIB::ZLIB)

find_package(BZip2 REQUIRED)
target_link_libraries(abc PUBLIC BZip2::BZip2)

if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
  target_compile_options(abc
    PUBLIC
    -Wno-write-strings
    -Wno-narrowing
  )
elseif (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
  target_compile_options(abc
    PUBLIC
    -Wno-implicit-const-int-float-conversion
    -Wno-parentheses-equality
    -Wno-string-plus-char
    -Wno-writable-strings
  )
elseif (CMAKE_CXX_COMPILER_ID STREQUAL "MSVC")
  target_compile_options(abc
    PUBLIC
    /wd4018
    /wd4267
    /wd4311
    /wd4312
    /wd4334
    /wd4477
    /wd4996
    /wd4997
  )
endif()

add_subdirectory(src)

if(ABC_NAMESPACE)
  get_target_property(abc_SOURCES abc SOURCES)
  foreach(src_file ${abc_SOURCES})
    get_filename_component(src_ext ${src_file} EXT)
    if(${src_ext} STREQUAL ".c")
      set_source_files_properties(${src_file} PROPERTIES LANGUAGE CXX)
    endif()
  endforeach()
  target_compile_definitions(abc PUBLIC ABC_NAMESPACE=${ABC_NAMESPACE})
endif()

add_executable(abc_exe)
set_property(TARGET abc_exe PROPERTY OUTPUT_NAME abc)
target_sources(abc_exe PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src/base/main/main.c)
if(ABC_NAMESPACE)
  set_source_files_properties(${CMAKE_CURRENT_SOURCE_DIR}/src/base/main/main.c PROPERTIES LANGUAGE CXX)
endif()
target_link_libraries(abc_exe PRIVATE abc)
