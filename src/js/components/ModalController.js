import React from 'react';
import { connect } from 'react-redux';
import Modal from 'react-modal';


Modal.setAppElement('#root');


const ModalController = (props) => {
  if (!props.direct && !props.child) return null;

  if (props.direct) return (
    <Modal isOpen={true} className="modal">
      {props.direct}
    </Modal>
  );

  return (
    <Modal isOpen={true} onRequestClose={props.close} className="modal">
      <span className="close" onClick={props.close}>
        <i className="icon ion-ios-close-empty"></i>
      </span>
      {props.child}
    </Modal>
  );
};


export default connect(
  state => ({
    child: state.modal.child,
  }),
  dispatch => ({
    close: () => dispatch({ type: 'MODAL::HIDE' }),
  }),
)(ModalController);
