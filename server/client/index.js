import init, { run_app } from './pkg/client.js';
async function main() {
   await init('/client/pkg/client_bg.wasm');
   run_app();
}
main()
