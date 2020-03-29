import init, { run_app } from './pkg/tasks.js';
async function main() {
   await init('tasks/pkg/tasks_bg.wasm');
   run_app();
}
main()
