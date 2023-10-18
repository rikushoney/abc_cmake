include(FindPackageHandleStandardArgs)

set(Readline_FOUND false)

find_library(Readline_LIBRARY NAMES readline)
find_path(Readline_INCLUDE_DIR NAMES readline/readline.h)

find_package_handle_standard_args(Readline
  REQUIRED_VARS
  Readline_LIBRARY
  Readline_INCLUDE_DIR
)

if (Readline_FOUND)
  mark_as_advanced(Readline_LIBRARY)
  
  add_library(Readline::Readline UNKNOWN IMPORTED)
  set_property(TARGET Readline::Readline
    PROPERTY
    IMPORTED_LOCATION ${Readline_LIBRARY}
  )
  target_include_directories(Readline::Readline
    INTERFACE
    ${Readline_INCLUDE_DIR}
  )
endif()
