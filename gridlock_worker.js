import { default as init } from './pkg/gridlock.js';

    var w=await init('pkg/gridlock_bg.wasm');


    await w.worker_entry();
