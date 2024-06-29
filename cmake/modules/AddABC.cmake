function(abc_add_library libname)
  cmake_parse_arguments(ARG
    ""
    ""
    "SOURCES"
    ${ARGN})

  add_library(${libname} STATIC)

  set_property(GLOBAL
    APPEND
    PROPERTY ABC_STATIC_LIBS
    ${libname})

  foreach(src ${ARG_SOURCES})
    cmake_path(GET src EXTENSION src_ext)
    if("${src_ext}" STREQUAL ".c")
      list(APPEND c_sources "${src}")
    elseif("${src_ext}" STREQUAL ".cpp")
      list(APPEND cxx_sources "${src}")
    endif()
  endforeach()

  if(NOT DEFINED c_sources AND NOT DEFINED cxx_sources)
    message(FATAL_ERROR "No C/CXX sources given to abc_add_library")
  endif()

  get_property(ABC_INCLUDE_DIRS
    GLOBAL
    PROPERTY ABC_INCLUDE_DIRS)

  get_property(ABC_COMPILE_DEFS
    GLOBAL
    PROPERTY ABC_COMPILE_DEFS)

  get_property(ABC_COMPILE_FLAGS
    GLOBAL
    PROPERTY ABC_COMPILE_FLAGS)

  get_property(ABC_C_STANDARD
    GLOBAL
    PROPERTY ABC_REQUIRED_C_STANDARD)

  get_property(ABC_CXX_STANDARD
    GLOBAL
    PROPERTY ABC_REQUIRED_CXX_STANDARD)

  set(c_libname "${libname}C")
  set(cxx_libname "${libname}CXX")

  if(DEFINED c_sources)
    add_library(${c_libname}
      OBJECT
      ${c_sources})

    get_property(ABC_C_STANDARD
      GLOBAL
      PROPERTY ABC_REQUIRED_C_STANDARD)

    set_target_properties(${c_libname}
      PROPERTIES
      C_STANDARD_REQUIRED 1
      C_STANDARD "${ABC_C_STANDARD}")

    target_include_directories(${c_libname}
      PRIVATE ${ABC_INCLUDE_DIRS})

    target_compile_definitions(${c_libname}
      PRIVATE ${ABC_COMPILE_DEFS})

    target_compile_options(${c_libname}
      PRIVATE ${ABC_COMPILE_FLAGS})
  endif()

  if(DEFINED cxx_sources)
    add_library(${cxx_libname}
      OBJECT
      ${cxx_sources})

    get_property(ABC_CXX_STANDARD
      GLOBAL
      PROPERTY ABC_REQUIRED_CXX_STANDARD)

    set_target_properties(${cxx_libname}
      PROPERTIES
      CXX_STANDARD_REQUIRED 1
      CXX_STANDARD "${ABC_CXX_STANDARD}")

    target_include_directories(${cxx_libname}
      PRIVATE ${ABC_INCLUDE_DIRS})

    target_compile_definitions(${cxx_libname}
      PRIVATE ${ABC_COMPILE_DEFS})

    target_compile_options(${cxx_libname}
      PRIVATE ${ABC_COMPILE_FLAGS})
  endif()

  if(TARGET ${c_libname})
    target_sources(${libname}
      PRIVATE $<TARGET_OBJECTS:${c_libname}>)
  endif()

  if(TARGET ${cxx_libname})
    target_sources(${libname}
      PRIVATE $<TARGET_OBJECTS:${cxx_libname}>)
  endif()

  set_target_properties(${libname}
    PROPERTIES
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/lib"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/lib")
endfunction()
