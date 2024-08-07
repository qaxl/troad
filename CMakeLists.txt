cmake_minimum_required(VERSION 3.25)
project(troad)

# Adjust the C++ standard.
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_EXTENSIONS ON)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Tidy up our build directory. Have all executables in build/bin and library and temp files in build/lib.
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Basically, forces the use of mold. (Very fast linker :))
option(WITH_MOLD_AS_LINKER "Whether use mold as linker or not (use the system default instead). If enabled, will use mold." ON)
option(WITH_NATIVE_OPTIMIZATIONS "Whether enable native optimizations. (Not recommended for distributed executables!)" ${${CMAKE_BUILD_TYPE} STREQUAL "RelWithDebInfo"})
option(WITH_DEBUG_ASAN "Whether to enable Address Sanitizer on Debug builds, if available." ON)

if (WITH_NATIVE_OPTIMIZATIONS)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -march=native -mtune=native")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -march=native -mtune=native")

    message("- Enabling native optimizations.")
    message("- Please disable, when distributing!!")
endif()

if (WITH_MOLD_AS_LINKER)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fuse-ld=mold")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fuse-ld=mold")

    message("- Linking with mold.")
endif()

if (WITH_DEBUG_ASAN)
    set (CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -fno-omit-frame-pointer -fsanitize=address")
    set (CMAKE_LINKER_FLAGS_DEBUG "${CMAKE_LINKER_FLAGS_DEBUG} -fno-omit-frame-pointer -fsanitize=address")

    message("- Enabling ASan.")
endif()

add_subdirectory(src)
add_subdirectory(third_party)

