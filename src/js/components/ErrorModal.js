import React from 'react';


const ErrorModal = (props) => {
  return (
    <div className="modal-body">
      <h2>Error</h2>
      <p>
        {props.msg || "Unexpected zmachine error:"}
      </p>
      <pre className="danger">
        {props.err.stack}
      </pre>
    </div>
  );
};


export default ErrorModal;
