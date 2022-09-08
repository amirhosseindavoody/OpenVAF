use crate::spec::{linux_base, Target};

pub fn target() -> Target {
    Target {
        llvm_target: "aarch64-unknown-linux-gnu".to_string(),
        pointer_width: 64,
        data_layout: "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128".to_string(),
        arch: "aarch64".to_string(),
        options: linux_base::opts(),
    }
}
