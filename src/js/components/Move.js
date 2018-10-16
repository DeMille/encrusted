import React, { Component } from 'react';
import { connect } from 'react-redux';

import measure from '../measure';
import Listener from '../Listener';
import Spoken from './Spoken';


function decodeEntities(str) {
  const textarea = measure._textarea || document.createElement('textarea');
  if (!measure._textarea) measure._textarea = textarea;

  textarea.innerHTML = str;
  return textarea.value;
}


class Move extends Component {
  constructor(props) {
    super(props);

    this.onBlur = this.onBlur.bind(this);
    this.onKey = this.onKey.bind(this);
    this.listener = null;

    this.state = { index: props.history.length };
  }

  onBlur() {
    if (this.inputEl && this.props.isActive) {
      setTimeout(() => this.inputEl.focus());
    }
  }

  onKey(ev) {
    // enter key submits the user input
    if (ev.keyCode === 13) {
      this.props.submit(this.inputEl.value);
      this.inputEl.readonly = true;
    }

    // up key goes backwards through command history
    if (ev.keyCode === 38) {
      let index = this.state.index;

      if (index > 0) {
        index -= 1;
        this.setState({ index });
      }

      const cmd = this.props.history[index];

      if (cmd) {
        ev.preventDefault();
        this.inputEl.value = cmd;
        this.inputEl.setSelectionRange(cmd.length, cmd.length);
      }
    }

    // down key goes forwards through command history
    if (ev.keyCode === 40) {
      let index = this.state.index;

      if (index < this.props.history.length - 1) {
        index += 1;
        this.setState({ index });
      }

      const cmd = this.props.history[index];

      if (cmd) {
        ev.preventDefault();
        this.inputEl.value = cmd;
        this.inputEl.setSelectionRange(cmd.length, cmd.length);
      }
    }

    // ctrl+s to save
    if (ev.keyCode === 83 && ev.ctrlKey) {
      ev.preventDefault();
      this.inputEl.value = 'save';
      this.props.submit(this.inputEl.value);
    }

    // ctrl+o to "open" save file
    if (ev.keyCode === 79 && ev.ctrlKey) {
      ev.preventDefault();
      this.inputEl.value = 'restore';
      this.props.submit(this.inputEl.value);
    }

    // ctrl+z to undo, ctrl+shift+z to redo
    if (ev.keyCode === 90 && ev.ctrlKey && ev.shiftKey) {
      ev.preventDefault();
      this.props.redo();
    } else if (ev.keyCode === 90 && ev.ctrlKey) {
      ev.preventDefault();
      this.props.undo();
    }
  }

  calculateWidth(text) {
    // get pixel length of the last line to set the width of the input
    const line = text
      .split('<br>')
      .pop()
      .replace(/<span>/g, '')
      .replace(/<\/span>/g, '');

    const width = measure(decodeEntities(line), '15px Lora');
    return `calc(100% - ${width}px - 6px)`;
  }

  focus() {
    this.inputEl && requestAnimationFrame(() => {
      const len = this.inputEl.value.length;

      this.inputEl.focus(); // forces reflow
      this.inputEl.setSelectionRange(len, len);
    });
  }

  updateListener() {
    const remove = () => {
      if (this.listener) {
        this.listener.stop();
        this.listener = null;
      }
    };

    // voice commands not enabled or were just turned off
    if (!this.props.voiceCommandsEnabled) {
      remove();
      return;
    }

    // not available on this platform
    if (!Listener.isAvailable) return;

    // stop listener when inactive / out of focus
    if (!this.props.isActive || this.props.modalOpen) {
      remove();
      return;
    }

    // otherwise, start listening for commands
    if (!this.listener) {
      this.listener = new Listener((result) => {
        remove();

        if (result === 'redo') {
          this.props.redo();
        } else if (result === 'undo') {
          this.props.undo();
        } else {
          this.inputEl.value = result;
          this.props.submit(this.inputEl.value);
        }
      });
    }

    this.listener.start();
  }

  shouldComponentUpdate(next) {
    return (
      next.quit ||
      next.isActive !== this.props.isActive ||
      this.props.isActive && (next.modalOpen !== this.props.modalOpen)
    );
  }

  componentDidUpdate() {
    this.props.isActive && this.focus();
    this.updateListener();
  }

  componentDidMount() {
    this.focus();
    this.updateListener();
  }

  render() {
    const width = this.calculateWidth(this.props.text);
    const active = this.props.isActive && !this.props.modalOpen && !this.props.quit;

    return (
      <div className="move">
        <Spoken isEnabled={this.props.isRead}>
          <span dangerouslySetInnerHTML={{ __html: this.props.text }} />
        </Spoken>

        <span className="user-input" style={{ display: (active) ? 'none' : null }}>
          {this.props.input}
        </span>

        <input
          defaultValue={this.props.input}
          readOnly={!active}
          style={{ width, display: (active) ? null : 'none' }}
          ref={el => this.inputEl = el}
          onKeyDown={this.onKey}
          onBlur={this.onBlur}
        />
      </div>
    );
  }
}


export default connect(
  state => ({
    modalOpen: !!state.modal.child,
    voiceCommandsEnabled: state.settings.listen,
    quit: !!state.transcript.quit,
  }),
)(Move);
