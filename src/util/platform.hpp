#pragma once

#if defined(WIN32) || defined(_WIN32) || defined(__WIN32__) || defined(__NT__)
#define TROAD_WIN32
#ifndef _WIN64
#error \
    "32-bit windows is unsupported. You may remove this error message, but compilation isn't guaranteed to work."
#endif
#elif __APPLE__
#include <TargetConditionals.h>
#if defined(TARGET_IPHONE_SIMULATOR) || defined(TARGET_OS_IPHONE) || \
    defined(TARGET_OS_MACCATALYST)
#error \
    "Apple platforms other than macOS aren't officially supported. You may remove this and try compiling, but you're on your own."
#elif TARGET_OS_MAC
#define TROAD_MACOS
#define TROAD_POSIX
#else
#error "Unknown Apple platform"
#endif
#elif defined(__linux__) || defined(__ANDROID__)
#define TROAD_LINUX
#define TROAD_POSIX
#elif __unix__  // all unices not caught above
#define TROAD_UNIX
#define TROAD_POSIX
#elif defined(_POSIX_VERSION)
#define TROAD_POSIX
#else
#error "Unknown compiler"
#endif
