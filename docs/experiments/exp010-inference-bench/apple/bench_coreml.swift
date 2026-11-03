// exp010 · Core ML 逐帧基准（GTCRN），输出一行 JSON，字段同 rustbe 的 bench。
//
// 用法：
//   bench_coreml bench <模型 .mlpackage|.mlmodelc> <cpu|gpu|ane|all> <in.f32> <ort.f32> [--frames N] [--warmup N] [--parity-frames N] [--state]
//   bench_coreml plan  <模型 .mlpackage|.mlmodelc> <cpu|gpu|ane|all>      每个算子落在哪个计算单元（MLComputePlan）
//
// in.f32 / ort.f32：每帧 514 个小端 f32（re[257] 再 im[257]），来自 rustbe bench 的 --dump-io。
// 同 rustbe 口径：第 0 帧是冷启动帧，接着预热帧（默认 200），再计时帧（默认 6000）；转储本身已按 mix.wav 循环排好。
// 逐值一致只比前 --parity-frames 帧（默认 610 = 整段 mix.wav 不循环），算法同 rustbe parity：最大绝对差、整段 SNR、首个差 > 1e-3 的位置。
// 实时做法：输入输出 MLMultiArray 全部预分配（outputBackings），两组缓存来回交换，每帧只 memcpy 频谱。
import CoreML
import Foundation

let BINS = 257
let PER = 2 * BINS

func die(_ s: String) -> Never { FileHandle.standardError.write((s + "\n").data(using: .utf8)!); exit(2) }

func readF32(_ p: String) -> [Float] {
    guard let d = FileManager.default.contents(atPath: p) else { die("读不到 \(p)") }
    return d.withUnsafeBytes { Array($0.bindMemory(to: Float.self)) }
}

func units(_ s: String) -> MLComputeUnits {
    switch s {
    case "cpu": return .cpuOnly
    case "gpu": return .cpuAndGPU
    case "ane": return .cpuAndNeuralEngine
    case "all": return .all
    default: die("计算单元只收 cpu|gpu|ane|all")
    }
}

func nowMs() -> Double { Double(DispatchTime.now().uptimeNanoseconds) / 1e6 }
func cpuSeconds() -> Double {
    var r = rusage()
    getrusage(RUSAGE_SELF, &r)
    return Double(r.ru_utime.tv_sec + r.ru_stime.tv_sec) + Double(r.ru_utime.tv_usec + r.ru_stime.tv_usec) / 1e6
}
func pct(_ v: [Double], _ q: Double) -> Double { v[min(v.count - 1, Int((Double(v.count) * q).rounded(.down)))] }

func compiledURL(_ p: String) throws -> (URL, Double) {
    let u = URL(fileURLWithPath: p)
    if u.pathExtension == "mlmodelc" { return (u, 0) }
    let t = nowMs()
    let c = try MLModel.compileModel(at: u)
    return (c, nowMs() - t)
}

func arr(_ shape: [Int], _ dt: MLMultiArrayDataType) throws -> MLMultiArray {
    let a = try MLMultiArray(shape: shape.map { NSNumber(value: $0) }, dataType: dt)
    // 清零
    _ = a.withUnsafeMutableBytes { p, _ in memset(p.baseAddress!, 0, p.count) }
    return a
}

// mix [1,257,1,2]：内存里是 bin 主序、re/im 交错。按 strides 写，别假设连续。
func writeMix(_ a: MLMultiArray, _ src: UnsafePointer<Float>) {
    let s1 = a.strides[1].intValue, s3 = a.strides[3].intValue
    if a.dataType == .float32 {
        a.withUnsafeMutableBufferPointer(ofType: Float.self) { p, _ in
            for k in 0..<BINS { p[k * s1] = src[k]; p[k * s1 + s3] = src[BINS + k] }
        }
    } else {
        a.withUnsafeMutableBufferPointer(ofType: Float16.self) { p, _ in
            for k in 0..<BINS { p[k * s1] = Float16(src[k]); p[k * s1 + s3] = Float16(src[BINS + k]) }
        }
    }
}
func readEnh(_ a: MLMultiArray, _ dst: UnsafeMutablePointer<Float>) {
    let s1 = a.strides[1].intValue, s3 = a.strides[3].intValue
    if a.dataType == .float32 {
        a.withUnsafeBufferPointer(ofType: Float.self) { p in
            for k in 0..<BINS { dst[k] = p[k * s1]; dst[BINS + k] = p[k * s1 + s3] }
        }
    } else {
        a.withUnsafeBufferPointer(ofType: Float16.self) { p in
            for k in 0..<BINS { dst[k] = Float(p[k * s1]); dst[BINS + k] = Float(p[k * s1 + s3]) }
        }
    }
}

func cmdPlan(_ a: [String]) async throws {
    let (url, _) = try compiledURL(a[0])
    let cfg = MLModelConfiguration()
    cfg.computeUnits = units(a[1])
    let plan = try await MLComputePlan.load(contentsOf: url, configuration: cfg)
    guard case let .program(prog) = plan.modelStructure, let main = prog.functions["main"] else { die("不是 mlprogram") }
    var count: [String: Int] = [:]
    var cost: [String: Double] = [:]
    var rows: [String] = []
    func walk(_ ops: [MLModelStructure.Program.Operation]) {
        for op in ops {
            var dev = "none"
            if let u = plan.deviceUsage(for: op) {
                switch u.preferred {
                case .cpu: dev = "cpu"
                case .gpu: dev = "gpu"
                case .neuralEngine: dev = "ane"
                @unknown default: dev = "?"
                }
            }
            let w = plan.estimatedCost(of: op)?.weight ?? 0
            count[dev, default: 0] += 1
            cost[dev, default: 0] += w
            if dev != "none" { rows.append("\(op.operatorName)\t\(dev)\t\(String(format: "%.4f", w))") }
            for b in op.blocks { walk(b.operations) }
        }
    }
    walk(main.block.operations)
    FileHandle.standardError.write((rows.joined(separator: "\n") + "\n").data(using: .utf8)!)
    let c = count.map { "\"\($0.key)\":\($0.value)" }.sorted().joined(separator: ",")
    let w = cost.map { "\"\($0.key)\":\(String(format: "%.4f", $0.value))" }.sorted().joined(separator: ",")
    print("{\"model\":\"\(URL(fileURLWithPath: a[0]).lastPathComponent)\",\"units\":\"\(a[1])\",\"ops_by_device\":{\(c)},\"cost_by_device\":{\(w)}}")
}

func cmdBench(_ a: [String]) throws {
    var frames = 6000, warmup = 200, parityN = 610, stateful = false
    var i = 4
    while i < a.count {
        switch a[i] {
        case "--frames": frames = Int(a[i + 1])!; i += 2
        case "--warmup": warmup = Int(a[i + 1])!; i += 2
        case "--parity-frames": parityN = Int(a[i + 1])!; i += 2
        case "--state": stateful = true; i += 1
        default: die("不认识 \(a[i])")
        }
    }
    let input = readF32(a[2]), ref = readF32(a[3])
    let nIn = input.count / PER
    guard nIn > 0, ref.count == input.count else { die("in/ort 帧数不对") }

    vdn_alloc_hook_install()
    let t0 = nowMs()
    let (url, compileMs) = try compiledURL(a[0])
    let cfg = MLModelConfiguration()
    cfg.computeUnits = units(a[1])
    let model = try MLModel(contentsOf: url, configuration: cfg)
    let openMs = nowMs() - t0

    let desc = model.modelDescription
    let mixDT = desc.inputDescriptionsByName["mix"]!.multiArrayConstraint!.dataType
    let mix = try arr([1, BINS, 1, 2], mixDT)
    let enhOut = try arr([1, BINS, 1, 2], desc.outputDescriptionsByName["enh"]!.multiArrayConstraint!.dataType)

    // 显式缓存：两组 A/B 交替（A 进 B 出，下一帧 B 进 A 出），零拷贝
    let cacheNames = ["conv_cache", "tra_cache", "inter_cache"]
    var provs: [MLFeatureProvider] = []
    var opts: [MLPredictionOptions] = []
    var state: MLState? = nil
    if stateful {
        state = model.makeState()
        provs = [try MLDictionaryFeatureProvider(dictionary: ["mix": mix])]
        let o = MLPredictionOptions(); o.outputBackings = ["enh": enhOut]; opts = [o]
    } else {
        var sets: [[String: MLMultiArray]] = [[:], [:]]
        for n in cacheNames {
            let c = desc.inputDescriptionsByName[n]!.multiArrayConstraint!
            let shape = c.shape.map { $0.intValue }
            sets[0][n] = try arr(shape, c.dataType)
            sets[1][n] = try arr(shape, c.dataType)
        }
        for k in 0..<2 {
            var d: [String: Any] = ["mix": mix]
            for n in cacheNames { d[n] = sets[k][n]! }
            provs.append(try MLDictionaryFeatureProvider(dictionary: d))
            let o = MLPredictionOptions()
            var ob: [String: Any] = ["enh": enhOut]
            for n in cacheNames { ob[n + "_out"] = sets[1 - k][n]! }
            o.outputBackings = ob
            opts.append(o)
        }
    }

    var out = [Float](repeating: 0, count: PER)
    var times: [Double] = []
    times.reserveCapacity(frames)
    var firstMs = 0.0, nonfinite = 0
    var maxAbs = 0.0, sig = 0.0, err = 0.0, worst = 0
    var firstBad = "None"
    var a0: UInt64 = 0, c0 = 0.0, w0 = 0.0
    let total = 1 + warmup + frames
    let t0w = 1 + warmup
    for f in 0..<total {
        if f == t0w { a0 = vdn_alloc_count(); c0 = cpuSeconds(); w0 = nowMs() }
        let k = f % nIn
        input.withUnsafeBufferPointer { writeMix(mix, $0.baseAddress! + k * PER) }
        let p = stateful ? 0 : f & 1
        let t = nowMs()
        try autoreleasepool {
            let r: MLFeatureProvider
            if let s = state { r = try model.prediction(from: provs[0], using: s, options: opts[0]) }
            else { r = try model.prediction(from: provs[p], options: opts[p]) }
            _ = r
        }
        let dt = nowMs() - t
        if f == 0 { firstMs = dt }
        out.withUnsafeMutableBufferPointer { readEnh(enhOut, $0.baseAddress!) }
        if f >= t0w { times.append(dt) }
        if !out.allSatisfy({ $0.isFinite }) && f >= t0w { nonfinite += 1 }
        if f < min(parityN, nIn) {  // 逐值一致：从第 0 帧起（缓存从零开始，和 ORT 转储同起点）
            for j in 0..<PER {
                let x = Double(ref[f * PER + j]), y = Double(out[j])
                let d = abs(x - y)
                sig += x * x; err += d * d
                if d > maxAbs { maxAbs = d; worst = f }
                if d > 1e-3 && firstBad == "None" { firstBad = "Some((\(f), \(j % BINS)))" }
            }
        }
    }
    let wall = (nowMs() - w0) / 1e3
    let cpu = cpuSeconds() - c0
    let allocs = Double(vdn_alloc_count() - a0) / Double(frames)
    let mean = times.reduce(0, +) / Double(times.count)
    times.sort()
    let snr = 10 * log10(sig / max(err, 1e-300))
    let backend = "coreml-\(a[1])" + (stateful ? "-state" : "") + (mixDT == .float16 ? "-fp16io" : "")
    let f4 = { (x: Double) in String(format: "%.4f", x) }
    print("{\"model\":\"gtcrn\",\"backend\":\"\(backend)\",\"artifact\":\"\(URL(fileURLWithPath: a[0]).lastPathComponent)\",\"frames\":\(frames),\"warmup\":\(warmup),\"p50_ms\":\(f4(pct(times, 0.5))),\"p99_ms\":\(f4(pct(times, 0.99))),\"p999_ms\":\(f4(pct(times, 0.999))),\"max_ms\":\(f4(times.last!)),\"mean_ms\":\(f4(mean)),\"compile_ms\":\(String(format: "%.2f", compileMs)),\"open_ms\":\(String(format: "%.2f", openMs)),\"first_frame_ms\":\(String(format: "%.3f", firstMs)),\"cpu_over_wall\":\(String(format: "%.3f", cpu / wall)),\"process_allocs_per_frame\":\(String(format: "%.3f", allocs)),\"nonfinite_frames\":\(nonfinite),\"parity_frames\":\(min(parityN, nIn, total)),\"parity_max_abs\":\(String(format: "%.3e", maxAbs)),\"parity_worst_frame\":\(worst),\"parity_snr_db\":\(String(format: "%.2f", snr)),\"parity_first_bad\":\"\(firstBad)\"}")
}

let argv = Array(CommandLine.arguments.dropFirst())
do {
    switch argv.first {
    case "bench": try cmdBench(Array(argv.dropFirst()))
    case "plan":
        let sem = DispatchSemaphore(value: 0)
        var e: Error? = nil
        Task { do { try await cmdPlan(Array(argv.dropFirst())) } catch { e = error }; sem.signal() }
        sem.wait()
        if let e { throw e }
    default: die("用法见 bench_coreml.swift 顶部")
    }
} catch { die("出错：\(error)") }
