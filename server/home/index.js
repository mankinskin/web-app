import init, { run_app } from './pkg/home.js';
async function main() {
   await init('/home/pkg/home_bg.wasm');
   run_app();
}
main()
