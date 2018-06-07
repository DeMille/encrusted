const EventEmitter = require('events').EventEmitter;

class WorkerWrapper extends EventEmitter {
  constructor(url) {
    super();
    this._worker = new Worker(url);
    this._worker.onmessage = ev => this.emit(ev.data.type, ev.data.msg);
  }

  send(type, msg, transfer) {
    this._worker.postMessage({ type, msg }, transfer);
  }

  sendAnd(type, msg, transfer) {
    return new Promise((resolve) => {
      this.once(type, value => resolve(value));
      this._worker.postMessage({ type, msg }, transfer);
    });
  }

  load(filename, file) {
    this.send('load', { filename, file }, [file]);
  }

  terminate() {
    this._worker.terminate();
  }
}

export default WorkerWrapper;
