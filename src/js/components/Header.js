import React from 'react';
import { connect } from 'react-redux';


const Header = (props) => {
  let title = window.location.pathname.split('/').pop();
  title = title.charAt(0).toUpperCase() + title.slice(1);

  document.title = (props.left) ? `${title} - ${props.left}` : title;

  return (
    <div className="header">
      <div>
        <div className="left">
          {props.canUndo &&
            <i className="undo icon ion-chevron-left" onClick={props.undo}></i>
          }

          {props.left || '\u00A0'}
        </div>

        <div className="right">
          {props.canRedo &&
            <i className="redo icon ion-chevron-right" onClick={props.redo}></i>
          }
          {props.right || '\u00A0'}
        </div>
      </div>
    </div>
  );
};


export default connect(
  state => ({
    left: state.transcript.header.left,
    right: state.transcript.header.right,
    canUndo: !state.transcript.quit && state.transcript.moves.length > 1,
    canRedo: !state.transcript.quit && !!state.transcript.undos.length,
  }),
  dispatch => ({
    undo: () => dispatch({ type: 'TS::UNDO' }),
    redo: () => dispatch({ type: 'TS::REDO' }),
  }),
)(Header);
