cmake_minimum_required(VERSION 3.10)

project(abc)

add_executable(abc)
target_include_directories(abc PRIVATE ${CMAKE_SOURCE_DIR}/src)

add_subdirectory(src)