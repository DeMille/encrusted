import { Wrapper, rust } from 'wasm-ffi';


const wasmURL = (process.env.NODE_ENV === 'production')
  ? '/encrusted/web.wasm'
  : '/web.wasm';


// hold onto active file in case of restarts
let file = null;


function sendWorkerMessage(type, msg) {
  postMessage({ type, msg });
}


const zmachine = new Wrapper({
  hook: [],
  create: [null, ['number', 'number']],
  feed: [null, ['string']],
  step: ['bool'],
  undo: ['bool'],
  redo: ['bool'],
  get_updates: [],
  restore: [null, ['string']],
  load_savestate: [null, ['string']],
  enable_instruction_logs: [null, ['bool']],
  get_object_details: [rust.string, ['number']],
  flush_log: [],
});


zmachine.imports(wrap => ({
  env: {
    js_message: wrap('string', 'string', sendWorkerMessage),

    trace: wrap('string', (msg) => {
      const err = new Error(msg);

      setTimeout(() => {
        zmachine.flush_log();
      }, 200);

      postMessage({
        type: 'error',
        msg: { msg, stack: err.stack }
      });
    }),
  },
}));


function step() {
  const done = zmachine.step();
  if (done) sendWorkerMessage('quit');
}


function instantiate() {
  if (zmachine.exports) return Promise.resolve();
  return zmachine.fetch(wasmURL).then(() => zmachine.hook());
}


// dispatch handlers based on incoming messages
onmessage = (ev) => {
  // only want to compile/load the module once
  if (ev.data.type === 'instantiate') {
    instantiate().catch(err => setTimeout(() => {
      console.log('Error starting wasm: ', err, err.stack);
    }));
  }

  if (ev.data.type === 'load') {
    instantiate()
      .then(() => {
        file = new Uint8Array(ev.data.msg.file);
        const file_ptr = zmachine.utils.writeArray(file);

        zmachine.create(file_ptr, file.length);
        sendWorkerMessage('loaded');
      })
      .catch(err => setTimeout(() => {
        console.log('Error starting wasm: ', err, err.stack);
      }));
  }

  if (ev.data.type === 'start') {
    step();
  }

  if (ev.data.type === 'restart') {
    const file_ptr = zmachine.utils.writeArray(file);

    zmachine.create(file_ptr, file.length);
    sendWorkerMessage('loaded');
  }

  if (ev.data.type === 'input') {
    zmachine.feed(ev.data.msg);
    step();
  }

  if (ev.data.type === 'restore') {
    zmachine.restore(ev.data.msg);
    step();
  }

  if (ev.data.type === 'load_savestate') {
    zmachine.load_savestate(ev.data.msg);
    step();
    zmachine.feed('look'); // get description text and then undo
    step();
    zmachine.undo();
  }

  if (ev.data.type === 'undo') {
    const ok = zmachine.undo();

    sendWorkerMessage('undo', ok);
    zmachine.get_updates();
  }

  if (ev.data.type === 'redo') {
    const ok = zmachine.redo();

    sendWorkerMessage('redo', ok);
    zmachine.get_updates();
  }

  if (ev.data.type === 'enable:instructions') {
    zmachine.enable_instruction_logs(!!ev.data.msg);
  }

  if (ev.data.type === 'getDetails') {
    const str = zmachine.get_object_details(ev.data.msg);
    sendWorkerMessage('getDetails', str.value);
    str.free();
  }
};
