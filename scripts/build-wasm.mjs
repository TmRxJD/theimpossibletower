import { spawnSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const target = resolve(root, process.env.CARGO_TARGET_DIR || 'engine/target');
function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit', shell: false });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
run(process.env.CARGO || 'cargo', ['build', '--manifest-path', 'engine/Cargo.toml', '--target', 'wasm32-unknown-unknown', '--release', '--target-dir', target]);
mkdirSync(resolve(root, 'src/wasm'), { recursive: true });
run(process.env.WASM_BINDGEN || 'wasm-bindgen', [resolve(target, 'wasm32-unknown-unknown/release/impossible_tower_engine.wasm'), '--target', 'web', '--out-dir', 'src/wasm', '--out-name', 'impossible_tower']);
