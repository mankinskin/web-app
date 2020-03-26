import init, { run_app } from './pkg/server.js';
async function main() {
   await init('/pkg/server_bg.wasm');
   run_app();
}
main()
