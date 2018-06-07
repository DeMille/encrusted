import React from 'react';
import { connect } from 'react-redux';


const Restore = (props) => {
  let list;
  let el;

  const options = props.saves.reverse().map((save, index) => (
    <option value={save.data} key={index}>{save.id}</option>
  ));

  if (options.length) {
    list = (
      <div className="mb-4">
        <select name="restore" ref={ref => el = ref}>
          {options}
        </select>
      </div>
    );
  }

  return (
    <div>
      <h2>Restore from a save</h2>

      <div className="modal-body pt-4">
        {!list &&
          <p>
            You don't have any save files yet.
            <br/>
            You can save your progress by typing <span className="cmd">save</span>
            or with <span className="hotkey">ctrl + s</span>
          </p>
        }

        {list &&
          <p>
            Which save would you like to restore from?
          </p>
        }

        {list}
      </div>

      <div className="modal-footer">
        <button onClick={() => props.restore('')}>
          Cancel
        </button>

        {list &&
          <button className="inverted" onClick={() => props.restore(el.value)}>
            Restore
          </button>
        }
      </div>
    </div>
  );
};


export default connect(
  state => ({
    saves: state.saves.saves,
  }),
  dispatch => ({
    restore: data => dispatch({ type: 'SAVES::RESTORE', data }),
  }),
)(Restore);
