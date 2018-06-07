const Recognition = window.SpeechRecognition || window.webkitSpeechRecognition;

class Listener {
  constructor(cb) {
    this.cb = cb;
    this._stopped = true;
    this._rec = null;
  }

  start() {
    if (typeof Recognition === 'undefined') return this;

    if (this._rec) this._rec.abort();

    let error = false;
    this._stopped = false;
    this._rec = new Recognition();

    this._rec.onerror = (ev) => {
      if (ev.error !== 'no-speech') error = true;
    };

    this._rec.onend = () => {
      if (!this._stopped && !error) this._rec.start();
    };

    this._rec.onresult = (ev) => {
      let result = '';

      for (let i = ev.resultIndex; i < ev.results.length; ++i) {
        result += ev.results[i][0].transcript;
      }

      this.cb(result);
    };

    this._rec.start();

    return this;
  }

  stop() {
    this._stopped = true;
    if (this._rec) this._rec.abort();

    return this;
  }
}

Listener.isAvailable = (typeof Recognition !== 'undefined');

export default Listener;
