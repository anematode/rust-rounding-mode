# rust-rounding-mode
Implementation of floating-point rounding modes for Rust.

### Background

Rust does not have builtin facilities to control the rounding mode of floating-point operations—understandably so, since they're a fairly niche feature. The only interesting use I can think of is rigorous interval arithmetic, but the precision benefit is pretty negligible, all while taking up significantly more cycles changing the rounding mode. In fact, LLVM does not support rounding modes itself; C's `fesetround` accomplishes the task with inline assembly. Furthermore, compilers will
ignore rounding directives when doing constant folding at compile time.

Thus, the goal of this repository is to enable ergonomic access to this very niche feature, mostly for completeness' sake. Reliability is technically not guaranteed, as far as I can tell, but I don't think the compiler tinkers too much with inline `asm`.
