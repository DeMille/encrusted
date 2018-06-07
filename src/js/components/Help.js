import React from 'react';

const Help = () => {
  const title = window.location.pathname.split('/').pop();

  const url = ({
    zork1: 'http://infodoc.plover.net/manuals/zork1.pdf',
    zork2: 'http://infodoc.plover.net/manuals/zork2.pdf',
    zork3: 'http://infodoc.plover.net/manuals/zork3.pdf',
  })[title];

  return (
    <div className="modal-body">
      <h2>Information</h2>

      <h4 className="mt-0">Basics</h4>
      <p className="mb-0">
        Move around by typing the direction you want to go, like: <span className="cmd">go north</span> or <span className="cmd">go southeast</span>.
      </p>
      <p className="sm mb-2">
          (These can also be shortened to <span className="cmd">n</span> for north or <span className="cmd">se</span> for southeast)
      </p>
      <p>
        • Look around with <span className="cmd">look</span> <span className="sm">(or the shorthand: <span className="cmd">l</span>)</span>
        <br/>• Pick things up with <span className="cmd">take</span>
        <br/>• Check your inventory with <span className="cmd">i</span>
        <br/>• Move through your command history with <span className="hotkey">up</span>/<span className="hotkey">down</span>
      </p>

      <h4>Save / Loading</h4>
      <p className="mb-2">
        Your progress is automatically saved after each move and is restored on page load.
      </p>
      <p>
        • Save a specifc spot with the <span className="cmd">save</span> command or with <span className="hotkey">ctrl + s</span>
        <br/>• Choose a save to restore from with the <span className="cmd">restore</span> command or with <span className="hotkey">ctrl + o</span>
      </p>

      <h4>Undo / Redo</h4>
      <p>
        Use the arrows in the header to undo/redo a move or use: <span className="hotkey">ctrl + z</span> &amp; <span className="hotkey">ctrl + shift + z</span>.
      </p>

      <div>
        <h4>More</h4>
        <p>
          {!!url && <span>Read the original <a href={url} target="_blank">user manual</a><br /></span>}
          Use <span className="cmd">$help</span> to show available debug commands
        </p>
      </div>

      <p className="footer-link">
        <a href="https://github.com/demille/encrusted" target="_blank">
          Encrusted is open source. Find the project on <i className="icon ion-social-github"></i>.
        </a>
      </p>
    </div>
  );
};

export default Help;
