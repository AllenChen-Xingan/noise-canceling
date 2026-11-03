// 全进程 malloc 计数：挂 libmalloc 的 malloc_logger（Instruments 用的同一个钩子）。
// 数的是本进程所有线程（含 Core ML 内部线程）的分配；aned 等其他进程里的分配看不到。
#include <stdint.h>
void vdn_alloc_hook_install(void);
uint64_t vdn_alloc_count(void);
