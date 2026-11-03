#include "alloc_hook.h"
#include <stdatomic.h>

typedef void(malloc_logger_t)(uint32_t type, uintptr_t a1, uintptr_t a2, uintptr_t a3, uintptr_t result,
                              uint32_t skip);
extern malloc_logger_t *malloc_logger;  // libmalloc 导出的全局函数指针

static _Atomic uint64_t g_allocs;
#define LOG_ALLOCATE 2u  // MALLOC_LOG_TYPE_ALLOCATE

static void hook(uint32_t type, uintptr_t a1, uintptr_t a2, uintptr_t a3, uintptr_t result, uint32_t skip) {
    (void)a1; (void)a2; (void)a3; (void)result; (void)skip;
    if (type & LOG_ALLOCATE) atomic_fetch_add_explicit(&g_allocs, 1, memory_order_relaxed);
}
void vdn_alloc_hook_install(void) { malloc_logger = hook; }
uint64_t vdn_alloc_count(void) { return atomic_load_explicit(&g_allocs, memory_order_relaxed); }
