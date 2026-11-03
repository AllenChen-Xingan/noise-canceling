// exp010 · BNNS Graph 逐帧基准（GTCRN · FE-M），输出一行 JSON，字段同 bench_coreml / rustbe bench。
//
// 用法：bench_bnns <模型.mlmodelc> <in.f32> <ort.f32> [--frames N] [--warmup N] [--parity-frames N] [--multi] [--paced-ms 16]
//   模型：convert_*.py 转出的 fp32 mlpackage 用 xcrun coremlcompiler compile 编成 mlmodelc（或用 bench_coreml 首次编译的产物）
//   默认单线程编译（BNNSGraphCompileOptionsSetTargetSingleThread）；--multi 用默认多线程，对照用
// 做法：工作区按页对齐一次性分配；参数用裸指针；缓存两组 A/B 来回换指针，不拷贝；每帧只 memcpy 输入。
// 实时口径：计时只包 BNNSGraphContextExecute 一次调用。
#include <Accelerate/Accelerate.h>
#include <mach/mach_time.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <unistd.h>

#include "alloc_hook.h"

#define BINS 257
#define MAXARG 16

static double now_ms(void) {
    static mach_timebase_info_data_t tb;
    if (!tb.denom) mach_timebase_info(&tb);
    return (double)mach_absolute_time() * tb.numer / tb.denom / 1e6;
}
static double cpu_s(void) {
    struct rusage r;
    getrusage(RUSAGE_SELF, &r);
    return r.ru_utime.tv_sec + r.ru_stime.tv_sec + (r.ru_utime.tv_usec + r.ru_stime.tv_usec) / 1e6;
}
static float *read_f32(const char *p, size_t *n) {
    FILE *f = fopen(p, "rb");
    if (!f) { fprintf(stderr, "读不到 %s\n", p); exit(2); }
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    float *b = malloc(sz);
    if (fread(b, 1, sz, f) != (size_t)sz) exit(2);
    fclose(f);
    *n = sz / 4;
    return b;
}
static int cmpd(const void *a, const void *b) {
    double x = *(const double *)a, y = *(const double *)b;
    return x < y ? -1 : x > y;
}
static void on_msg(BNNSGraphMessageLevel lv, const char *msg, const char *loc, bnns_user_message_data_t *ctx) {
    (void)ctx;
    fprintf(stderr, "[bnns %u] %s %s\n", lv, msg, loc ? loc : "");
}

int main(int argc, char **argv) {
    if (argc < 4) { fprintf(stderr, "用法见 bench_bnns.c 顶部\n"); return 2; }
    int frames = 6000, warmup = 200, parity_n = 610, multi = 0;
    double paced = 0;
    for (int i = 4; i < argc; i++) {
        if (!strcmp(argv[i], "--frames")) frames = atoi(argv[++i]);
        else if (!strcmp(argv[i], "--warmup")) warmup = atoi(argv[++i]);
        else if (!strcmp(argv[i], "--parity-frames")) parity_n = atoi(argv[++i]);
        else if (!strcmp(argv[i], "--multi")) multi = 1;
        else if (!strcmp(argv[i], "--paced-ms")) paced = atof(argv[++i]);
        else { fprintf(stderr, "不认识 %s\n", argv[i]); return 2; }
    }
    size_t n_in, n_ref;
    float *input = read_f32(argv[2], &n_in), *ref = read_f32(argv[3], &n_ref);

    vdn_alloc_hook_install();
    double t0 = now_ms();
    bnns_graph_compile_options_t opt = BNNSGraphCompileOptionsMakeDefault();
    BNNSGraphCompileOptionsSetTargetSingleThread(opt, !multi);
    BNNSGraphCompileOptionsSetMessageLogCallback(opt, on_msg, NULL);
    BNNSGraphCompileOptionsSetMessageLogMask(opt, BNNSGraphMessageLevelUnsupported | BNNSGraphMessageLevelError |
                                                      BNNSGraphMessageLevelWarning);
    bnns_graph_t g = BNNSGraphCompileFromFile(argv[1], NULL, opt);
    BNNSGraphCompileOptionsDestroy(opt);
    if (!g.data) { fprintf(stderr, "BNNSGraphCompileFromFile 失败（看上面的 [bnns] 日志）\n"); return 3; }
    double compile_ms = now_ms() - t0;
    bnns_graph_context_t ctx = BNNSGraphContextMake(g);
    if (!ctx.data) { fprintf(stderr, "ContextMake 失败\n"); return 3; }
    BNNSGraphContextSetArgumentType(ctx, BNNSGraphArgumentTypePointer);
    size_t ws_size = BNNSGraphContextGetWorkspaceSize(ctx, NULL);
    long page = sysconf(_SC_PAGESIZE);
    size_t ws_alloc = (ws_size + page - 1) / page * page + page;
    void *ws = aligned_alloc(page, ws_alloc);
    memset(ws, 0, ws_alloc);
    double open_ms = now_ms() - t0;

    // 认模型：有 wav_in 就是 FE-M
    size_t nargs = BNNSGraphGetArgumentCount(g, NULL);
    if (nargs > MAXARG) { fprintf(stderr, "参数太多 %zu\n", nargs); return 3; }
    const char *names[MAXARG];
    BNNSGraphGetArgumentNames(g, NULL, nargs, names);
    int is_fem = BNNSGraphGetArgumentPosition(g, NULL, "wav_in") != SIZE_MAX;
    const char *in_name = is_fem ? "wav_in" : "mix", *out_name = is_fem ? "wav_out" : "enh";
    const int per = is_fem ? 160 : 2 * BINS;
    size_t n_frames_in = n_in / per;
    if (!n_frames_in || n_in != n_ref || n_in % per) { fprintf(stderr, "in/ort 帧数不对\n"); return 2; }

    // 缓存名配对
    const char *gt_c[] = {"conv_cache", "tra_cache", "inter_cache"};
    const char *gt_o[] = {"conv_cache_out", "tra_cache_out", "inter_cache_out"};
    char fem_c[6][16], fem_o[6][16];
    int nc = is_fem ? 6 : 3;
    const char *cin[6], *cout[6];
    for (int i = 0; i < nc; i++) {
        if (is_fem) {
            snprintf(fem_c[i], 16, "cache_in_%d", i);
            snprintf(fem_o[i], 16, "cache_out_%d", i);
            cin[i] = fem_c[i]; cout[i] = fem_o[i];
        } else { cin[i] = gt_c[i]; cout[i] = gt_o[i]; }
    }
    // 缓存大小：查不到形状接口时，用 ORT 一样的已知形状（元素个数）
    size_t gt_sz[] = {2 * 1 * 16 * 16 * 33, 2 * 3 * 1 * 1 * 16, 2 * 1 * 33 * 16};
    size_t csz[6];
    if (is_fem) {
        // FE-M 缓存形状由 convert_fem.json 给；这里要求通过环境变量 FEM_CACHE_SIZES="a,b,c,d,e,f"（元素数）传入
        const char *e = getenv("FEM_CACHE_SIZES");
        if (!e) { fprintf(stderr, "FE-M 需要 FEM_CACHE_SIZES（见 convert_fem.json 的 shapes）\n"); return 2; }
        for (int i = 0; i < 6; i++) { csz[i] = strtoul(e, (char **)&e, 10); if (*e == ',') e++; }
    } else memcpy(csz, gt_sz, sizeof gt_sz);

    float *xin = calloc(per, 4), *yout = calloc(per, 4);
    float *cache[2][6];
    for (int k = 0; k < 2; k++)
        for (int i = 0; i < nc; i++) cache[k][i] = calloc(csz[i], 4);

    bnns_graph_argument_t args[2][MAXARG];
    memset(args, 0, sizeof args);
    for (int k = 0; k < 2; k++) {
        size_t pi = BNNSGraphGetArgumentPosition(g, NULL, in_name), po = BNNSGraphGetArgumentPosition(g, NULL, out_name);
        if (pi == SIZE_MAX || po == SIZE_MAX) { fprintf(stderr, "找不到输入输出名\n"); return 3; }
        args[k][pi] = (bnns_graph_argument_t){.data_ptr = xin, .data_ptr_size = per * 4};
        args[k][po] = (bnns_graph_argument_t){.data_ptr = yout, .data_ptr_size = per * 4};
        for (int i = 0; i < nc; i++) {
            size_t a = BNNSGraphGetArgumentPosition(g, NULL, cin[i]), b = BNNSGraphGetArgumentPosition(g, NULL, cout[i]);
            if (a == SIZE_MAX || b == SIZE_MAX) { fprintf(stderr, "找不到缓存 %s/%s\n", cin[i], cout[i]); return 3; }
            args[k][a] = (bnns_graph_argument_t){.data_ptr = cache[k][i], .data_ptr_size = csz[i] * 4};
            args[k][b] = (bnns_graph_argument_t){.data_ptr = cache[1 - k][i], .data_ptr_size = csz[i] * 4};
        }
    }

    int total = 1 + warmup + frames, t0w = 1 + warmup;
    double *times = malloc(sizeof(double) * frames);
    double first_ms = 0, max_abs = 0, sig = 0, err = 0, w0 = 0, c0 = 0;
    int worst = 0, nonfinite = 0, bad_f = -1, bad_k = -1;
    uint64_t a0 = 0;
    for (int f = 0; f < total; f++) {
        if (f == t0w) { a0 = vdn_alloc_count(); c0 = cpu_s(); w0 = now_ms(); }
        if (paced > 0 && f >= t0w) {
            double wait = w0 + (f - t0w) * paced - now_ms();
            if (wait > 0) usleep((useconds_t)(wait * 1000));
        }
        const float *src = input + (size_t)(f % n_frames_in) * per;
        if (is_fem) memcpy(xin, src, per * 4);
        else for (int k = 0; k < BINS; k++) { xin[2 * k] = src[k]; xin[2 * k + 1] = src[BINS + k]; }  // [1,257,1,2] 交错
        double t = now_ms();
        int rc = BNNSGraphContextExecute(ctx, NULL, nargs, args[f & 1], ws_size, ws);
        double dt = now_ms() - t;
        if (rc) { fprintf(stderr, "执行失败 rc=%d 帧 %d\n", rc, f); return 4; }
        if (f == 0) first_ms = dt;
        if (f >= t0w) times[f - t0w] = dt;
        float o[2 * BINS];
        if (is_fem) memcpy(o, yout, per * 4);
        else for (int k = 0; k < BINS; k++) { o[k] = yout[2 * k]; o[BINS + k] = yout[2 * k + 1]; }
        int fin = 1;
        for (int j = 0; j < per; j++) fin &= isfinite(o[j]);
        if (!fin && f >= t0w) nonfinite++;
        if (f < parity_n && (size_t)f < n_frames_in) {
            for (int j = 0; j < per; j++) {
                double x = ref[(size_t)f * per + j], y = o[j], d = fabs(x - y);
                sig += x * x; err += d * d;
                if (d > max_abs) { max_abs = d; worst = f; }
                if (d > 1e-3 && bad_f < 0) { bad_f = f; bad_k = is_fem ? j : j % BINS; }
            }
        }
    }
    double wall = (now_ms() - w0) / 1e3, cpu = cpu_s() - c0;
    double allocs = (double)(vdn_alloc_count() - a0) / frames, mean = 0;
    for (int i = 0; i < frames; i++) mean += times[i];
    mean /= frames;
    qsort(times, frames, sizeof(double), cmpd);
#define P(q) times[(int)(frames * (q)) < frames ? (int)(frames * (q)) : frames - 1]
    char bad[48];
    if (bad_f < 0) snprintf(bad, sizeof bad, "None"); else snprintf(bad, sizeof bad, "Some((%d, %d))", bad_f, bad_k);
    int pn = parity_n < (int)n_frames_in ? parity_n : (int)n_frames_in;
    printf("{\"model\":\"%s\",\"backend\":\"bnns-graph%s%s\",\"frames\":%d,\"pace_ms\":%g,\"warmup\":%d,\"p50_ms\":%.4f,\"p99_ms\":%.4f,"
           "\"p999_ms\":%.4f,\"max_ms\":%.4f,\"mean_ms\":%.4f,\"compile_ms\":%.2f,\"open_ms\":%.2f,\"first_frame_ms\":%.3f,"
           "\"cpu_over_wall\":%.3f,\"process_allocs_per_frame\":%.3f,\"nonfinite_frames\":%d,\"workspace_bytes\":%zu,"
           "\"parity_frames\":%d,\"parity_max_abs\":%.3e,\"parity_worst_frame\":%d,\"parity_snr_db\":%.2f,\"parity_first_bad\":\"%s\"}\n",
           is_fem ? "fastenhancer_m" : "gtcrn", multi ? "-multi" : "", paced > 0 ? "-paced" : "", frames, paced, warmup, P(0.5),
           P(0.99), P(0.999), times[frames - 1], mean, compile_ms, open_ms, first_ms, cpu / wall, allocs, nonfinite, ws_size,
           pn, max_abs, worst, 10 * log10(sig / (err > 1e-300 ? err : 1e-300)), bad);
    return 0;
}
