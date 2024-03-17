use core::fmt;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use extism::*;
use extism_manifest::MemoryOptions;

fn bench_call_plugin_func(c: &mut Criterion) {
    {
        let source = Wasm::file(
            "./benches/extism/plugin_cs/bin/Release/net8.0/wasi-wasm/AppBundle/ExtismPlugin.wasm",
        );
        // let source = Wasm::file(
        //     "./benches/extism/plugin_cs/bin/Debug/net8.0/wasi-wasm/AppBundle/ExtismPlugin.wasm",
        // );
        // let data = Wasm::data(include_bytes!("plugin_cs/bin/Release/net8.0/wasi-wasm/AppBundle/ExtismPlugin.wasm"));
        let manifest = Manifest::new([source])
            .with_memory_options(MemoryOptions::new().with_max_var_bytes(5 * 1024 * 1024));
        let mut plugin = Plugin::new(&manifest, [], true).unwrap();

        let mut af_group = c.benchmark_group("ascii_buffer");

        for size in [(10, 10), (150, 80), (100, 100), (1000, 1000), (1920, 1080)].iter() {
            let name = format!("{}x{}", size.0, size.1);

            let rb = unsafe { core::mem::transmute::<_, u64>((size.0, size.1)) };

            struct Input(i32, i32);

            impl fmt::Display for Input {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}x{}", self.0, self.1)
                }
            }

            af_group.bench_with_input(
                BenchmarkId::from_parameter(Input(size.0, size.1)),
                size,
                |b, &size| {
                    b.iter(|| {
                        let buf = plugin.call::<u64, &[u8]>("rf_ascii_buffer", rb).unwrap();

                        assert_eq!(buf.len(), (size.0 * size.1) as usize);
                    });
                },
            );
        }

        drop(af_group);

        let mut mo_group = c.benchmark_group("mem_offset");

        for size in [(10, 10), (150, 80), (100, 100), (1000, 1000), (1920, 1080)].iter() {
            let name = format!("{}x{}", size.0, size.1);

            let rb = unsafe { core::mem::transmute::<_, u64>((size.0, size.1)) };

            struct Input(i32, i32);

            impl fmt::Display for Input {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}x{}", self.0, self.1)
                }
            }

            mo_group.bench_with_input(
                BenchmarkId::from_parameter(Input(size.0, size.1)),
                size,
                |b, &size| {
                    b.iter(|| {
                        let buf = plugin.call::<u64, &[u8]>("rf_mem_offset", rb).unwrap();

                        assert_eq!(buf.len(), (size.0 * size.1) as usize);
                    });
                },
            );
        }
    }
}

criterion_group!(benches, bench_call_plugin_func);
criterion_main!(benches);
