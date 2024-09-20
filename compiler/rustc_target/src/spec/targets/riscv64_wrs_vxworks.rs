use crate::spec::{base, StackProbeType, Target, TargetOptions};

pub(crate) fn target() -> Target {
    Target {
        llvm_target: "riscv64".into(),
        metadata: crate::spec::TargetMetadata {
            description: None,
            tier: Some(3),
            host_tools: Some(false),
            std: None, // STD is a work in progress for this target arch
        },
        pointer_width: 64,
        data_layout: "e-m:e-p:64:64-i64:64-i128:128-n32:64-S128".into(),
        arch: "riscv64".into(),
        options: TargetOptions {
            cpu: "generic-rv64".into(),
            llvm_abiname: "lp64d".into(),
            max_atomic_width: Some(64),
            features: "+m,+a,+f,+d,+c,+zicsr".into(),
            stack_probes: StackProbeType::Inline,
            ..base::vxworks::opts()
        },
    }
}
