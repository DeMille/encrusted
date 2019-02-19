import React, { Component } from 'react';
import { connect } from 'react-redux';


class Settings extends Component {
  constructor(props) {
    super(props);
    this.update = this.update.bind(this);
    this.clearMap = this.clearMap.bind(this);
    this.eraseAll = this.eraseAll.bind(this);
  }

  update(ev) {
    this.props.update(ev.target.name, ev.target.checked);

    if (ev.target.name === 'read' && !!ev.target.checked) {
      window.speechSynthesis.speak(new SpeechSynthesisUtterance(''));

      if (/iPad|iPhone|iPod/.test(navigator.userAgent)) {
        alert('Dictation enabled! You may need to re-enable it when you reload the page.');
      }
    }
  }

  deleteSaves() {
    const title = window.location.pathname.split('/').pop();
    const msg = `Are you sure you want to delete saves for ${title}?`;

    if (window.confirm(msg)) {
      localStorage.removeItem(`${title}::saves`);
      localStorage.removeItem(`${title}::savestate`);
    }
  }

  clearMap() {
    const title = window.location.pathname.split('/').pop();
    const msg = `Are you sure you want to clear the map for ${title}?`;

    if (window.confirm(msg)) {
      this.props.clearMap();
    }
  }

  eraseAll() {
    const title = window.location.pathname.split('/').pop();
    const msg = `Are you sure you want to delete all data and saves for ${title}?`;

    if (!window.confirm(msg)) return;

    Object.keys(localStorage).forEach((key) => {
      if (key.slice(0, title.length) === title) {
        localStorage.removeItem(key);
      }
    });

    this.props.restart();
    this.props.closeModal();
  }

  copy() {
    this.link.focus();
    this.link.select();

    try {
      document.execCommand('copy');
      this.ok.innerHTML = 'copied!';
    } catch (err) {
      console.error('Unable to copy: ', err);
    }
  }

  render() {
    const url = window.location.href.split('?').shift();
    const link = encodeURI(`${url}?save=${this.props.currentSave}`);

    return (
      <div className="modal-body">
        <h2>Configure</h2>

        <h4 className="mt-0 mb-2">
          Interface
        </h4>

        <div className="row align-center mb-1">
          <input
            type="checkbox"
            className="toggle"
            name="map"
            id="map"
            onChange={this.update}
            checked={!!this.props.map}
          />
          <label htmlFor="map"></label>

          <label htmlFor="map" className="mb-0">
            Show generated map
          </label>
        </div>

        <div className="row align-center mb-1">
          <input
            type="checkbox"
            className="toggle"
            name="tree"
            id="tree"
            onChange={this.update}
            checked={!!this.props.tree}
          />
          <label htmlFor="tree"></label>

          <label htmlFor="tree" className="mb-0">
            Show object tree
          </label>
        </div>

        <div className="row align-center mb-1">
          <input
            type="checkbox"
            className="toggle"
            name="plain"
            id="plain"
            onChange={this.update}
            checked={!!this.props.plain}
          />
          <label htmlFor="plain"></label>

          <label htmlFor="plain" className="mb-0">
            Plain output styling
          </label>
        </div>

        <div className="row align-center mb-1">
          <input
            type="checkbox"
            className="toggle"
            name="mono"
            id="mono"
            onChange={this.update}
            checked={!!this.props.mono}
          />
          <label htmlFor="mono"></label>

          <label htmlFor="mono" className="mb-0">
            Monospace font
          </label>
        </div>

        <div className="row align-center mb-1">
          <input
            type="checkbox"
            className="toggle"
            name="read"
            id="read"
            onChange={this.update}
            checked={!!this.props.read}
          />
          <label htmlFor="read"></label>

          <label htmlFor="read" className="mb-0">
            <i className="icon ion-android-volume-off label-icon unchecked"></i>
            <i className="icon ion-android-volume-up label-icon checked"></i> Narrate
          </label>
        </div>

        <div className="row align-center mb-1">
          <input
            type="checkbox"
            className="toggle"
            name="listen"
            id="listen"
            onChange={this.update}
            checked={!!this.props.listen}
          />
          <label htmlFor="listen"></label>

          <label htmlFor="listen" className="mb-0">
            <i className="icon ion-android-microphone-off label-icon unchecked"></i>
            <i className="icon ion-android-microphone label-icon checked"></i> Voice commands
            <span className="sm ml-2"> (chrome only)</span>
          </label>
        </div>

        <h4>
          Progress
        </h4>
        <p className="mb-1">
          Continue on another device:
          <span className="sm ml-2" ref={el => this.ok = el}></span>
        </p>

        <div className="row align-center">
          <p className="w-100 mb-1 row">
            <button className="group-left" onClick={() => this.copy()}>
              <i className="icon ion-clipboard"></i>
            </button>
            <input
              className="link"
              type="text"
              value={link}
              ref={el => this.link = el}
              readOnly
            />
          </p>
        </div>

        <h4 className="mb-2">
          Debug
        </h4>
        <div className="row align-center mb-1">
          <input
            type="checkbox"
            className="toggle"
            name="instructions"
            id="instructions"
            onChange={this.update}
            checked={!!this.props.instructions}
          />
          <label htmlFor="instructions"></label>

          <label htmlFor="instructions" className="mb-0">
            Show z-machine instruction log
          </label>
        </div>

        <h4>
          Resets
        </h4>
        <p className="sm">¡Danger!</p>

        <div className="row align-center">
          <button className="alert" onClick={this.clearMap}>
            <i className="icon ion-flame"></i> reset map
          </button>
          <button className="alert" onClick={this.deleteSaves}>
            <i className="icon ion-flame"></i> delete saves
          </button>
          <button className="danger" onClick={this.eraseAll}>
            <i className="icon ion-flame"></i> erase all &amp; restart
          </button>
        </div>
      </div>
    );
  }
}


export default connect(
  state => ({
    map: state.settings.map,
    tree: state.settings.tree,
    read: state.settings.read,
    plain: state.settings.plain,
    mono: state.settings.mono,
    listen: state.settings.listen,
    instructions: state.settings.instructions,
    currentSave: state.saves.current,
  }),
  dispatch => ({
    update: (name, value) => dispatch({ type: 'SETTING', name, value }),
    clearMap: () => dispatch({ type: 'MAP::CLEAR' }),
    restart: () => dispatch({ type: 'TS::RESTART' }),
    closeModal: () => dispatch({ type: 'MODAL::HIDE' }),
  }),
)(Settings);
