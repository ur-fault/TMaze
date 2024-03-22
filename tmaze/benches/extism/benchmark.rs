use core::fmt;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use extism::*;
use extism_manifest::MemoryOptions;

fn bench_render_frame(c: &mut Criterion) {
    {
        let source = Wasm::file(
            "./benches/extism/plugin_cs/bin/Release/net8.0/wasi-wasm/AppBundle/ExtismPlugin.wasm",
        );
        // let source = Wasm::file(
        //     "./benches/extism/plugin_cs/bin/Debug/net8.0/wasi-wasm/AppBundle/ExtismPlugin.wasm",
        // );
        let manifest = Manifest::new([source])
            .with_memory_options(MemoryOptions::new().with_max_var_bytes(5 * 1024 * 1024));
        let mut plugin = Plugin::new(&manifest, [], true).unwrap();

        let mut rf_group = c.benchmark_group("render_frame");

        struct Input(i32, i32);

        impl fmt::Display for Input {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}x{}", self.0, self.1)
            }
        }

        let sizes = [
            (10, 10),
            (150, 80),
            (250, 70),
            (100, 100),
            (1000, 1000),
            (1920, 1080),
        ];

        for size in sizes.iter() {
            let rb = unsafe { core::mem::transmute::<_, u64>((size.0, size.1)) };

            rf_group.bench_with_input(
                BenchmarkId::new("ascii_buffer", Input(size.0, size.1)),
                size,
                |b, &size| {
                    b.iter(|| {
                        let buf = plugin.call::<u64, &[u8]>("rf_ascii_buffer", rb).unwrap();

                        assert_eq!(buf.len(), (size.0 * size.1) as usize);
                    });
                },
            );
        }

        for size in sizes.iter() {
            let rb = unsafe { core::mem::transmute::<_, u64>((size.0, size.1)) };

            rf_group.bench_with_input(
                BenchmarkId::new("mem_offset", Input(size.0, size.1)),
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

criterion_group!(benches, bench_render_frame);
criterion_main!(benches);
