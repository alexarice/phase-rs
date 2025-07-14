(function() {
    var type_impls = Object.fromEntries([["gemm",[]],["gemm_common",[]],["nano_gemm",[]],["nano_gemm_c32",[]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[11,19,17,21]}