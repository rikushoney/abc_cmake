cmake_minimum_required(VERSION 3.10)

project(abc)

add_executable(abc)
target_include_directories(abc PRIVATE ${CMAKE_SOURCE_DIR}/src)
target_compile_definitions(abc PRIVATE ABC_USE_STDINT_H=1)

add_subdirectory(src)