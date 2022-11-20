#pragma once

#include <stdbool.h>

void __attribute__((cdecl)) long_mode_jump(void* address);

bool __attribute__((cdecl)) is_extended_cpuid_available(void);
bool __attribute__((cdecl)) is_cpuid_available(void);
