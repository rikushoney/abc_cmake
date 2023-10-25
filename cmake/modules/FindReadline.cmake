include(FindPackageHandleStandardArgs)

set(Readline_FOUND false)

if(NOT Readline_LIBRARY)
  find_library(Readline_LIBRARY NAMES readline)
endif()
if(NOT Readline_INCLUDE_DIR)
  find_path(Readline_INCLUDE_DIR NAMES readline/readline.h)
endif()

find_package_handle_standard_args(Readline
  REQUIRED_VARS
  Readline_LIBRARY
  Readline_INCLUDE_DIR
)

if(Readline_FOUND)
  if(NOT TARGET Readline::Readline)
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
endif()

mark_as_advanced(Readline_LIBRARY)
mark_as_advanced(Readline_INCLUDE_DIR)
