# Compute the installation prefix relative to this file
get_filename_component(_IMPORT_PREFIX "${CMAKE_CURRENT_LIST_FILE}" PATH)
get_filename_component(_IMPORT_PREFIX "${_IMPORT_PREFIX}" PATH)
get_filename_component(_IMPORT_PREFIX "${_IMPORT_PREFIX}" PATH)
get_filename_component(_IMPORT_PREFIX "${_IMPORT_PREFIX}" PATH)
if (_IMPORT_PREFIX STREQUAL "/")
	set(_IMPORT_PREFIX "")
endif()

add_library(my_cool_rust_pkg SHARED IMPORTED)
set_property(TARGET my_cool_rust_pkg PROPERTY IMPORTED_LOCATION "${_IMPORT_PREFIX}/lib/my_cool_rust_pkg/libmy_cool_rust_pkg.so")
set_property(TARGET my_cool_rust_pkg PROPERTY INTERFACE_INCLUDE_DIRECTORIES "${_IMPORT_PREFIX}/include/my_cool_rust_pkg")
