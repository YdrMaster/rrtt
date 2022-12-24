# rt-thread in rust

## version 0 (2022-12-22..)

直接翻译 C 版本试试看。

### 2022-12-24

C 里很常见的基于数值的条件编译：

```c
# if RT_THREAD_PRIORITY_MAX <= 32
...
#else
...
#endif
```

在 Rust 里没有对应。

想了一个办法，要求通过环境变量传入这种常量，在 build.rs 里把不同数值范围的环境变量转化为 [`rustc-cfg`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-cfgkeyvalue)。在 build.rs 里这样操作：

```rust
const PRIORITY_MAX: &str = "PRIORITY_MAX";
const PRIORITY_MAX_DEFAULT: usize = 8;
let val = match env::var(PRIORITY_MAX) {
    Ok(s) => match s.parse::<usize>() {
        Ok(val) => val,
        Err(_) => panic!("failed to parse env {PRIORITY_MAX}={s:?} for not a number."),
    },
    Err(env::VarError::NotPresent) => {
        println!("cargo:rustc-env={PRIORITY_MAX}={PRIORITY_MAX_DEFAULT}");
        PRIORITY_MAX_DEFAULT
    }
    Err(env::VarError::NotUnicode(_)) => {
        panic!("failed to parse env {PRIORITY_MAX} for not unicode.");
    }
};
println!("cargo:rerun-if-env-changed={PRIORITY_MAX}");
println!(
    "cargo:rustc-cfg={}",
    if val > 32 {
        "large_priority"
    } else {
        "small_priority"
    }
);
```

然后可以在代码里用 `#[cfg(large_priority)]` 或 `#[cfg(small_priority)]` 应用条件。
