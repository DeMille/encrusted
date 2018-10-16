import { combineReducers } from 'redux';


const initialTranscript = {
  moves: [],
  undos: [],
  history: [],
  header: { left: '', right: '' },
  quit: false,
};

function transcript(state = initialTranscript, action) {
  switch (action.type) {
    case 'TS::TEXT':
      return Object.assign({}, state, {
        moves: [...state.moves, { text: action.text, input: '' }],
        undos: [],
      });

    case 'TS::SUBMIT':
      const input = action.input.trim();
      const updated = Object.assign({}, state.moves.pop(), { input });

      const history = (input && state.history.slice(-1)[0] !== input)
        ? [...state.history, input]
        : state.history;

      return Object.assign({}, state, {
        moves: [...state.moves, updated],
        history,
      });

    case 'TS::UNDO':
      return Object.assign({}, state, {
        undos: [...state.undos, state.moves.pop()],
        moves: [...state.moves],
      });

    case 'TS::REDO':
      return Object.assign({}, state, {
        moves: [...state.moves, state.undos.pop()],
        undos: [...state.undos],
      });

    case 'TS::HEADER':
      const [left, right] = JSON.parse(action.data);

      return Object.assign({}, state, {
        header: { left, right },
      });

    case 'TS::STOP':
    case 'TS::RESTART':
      return Object.assign({}, initialTranscript);

    case 'TS::QUIT':
      return Object.assign({}, state, { quit: true });

    default:
      return state;
  }
}


const initialMap = {
  graph: null,
  update: {}, // hack, just a flag to force a D3 update
};

function map(state = initialMap, action) {
  switch (action.type) {
    case 'MAP::CREATE':
      return Object.assign({}, state, { graph: action.graph });

    case 'MAP::UPDATE':
      return Object.assign({}, state, { update: {} });

    default:
      return state;
  }
}


const initialTree = {
  data: '{}',
  getDetails: () => {},
};

function tree(state = initialTree, action) {
  switch (action.type) {
    case 'TREE::DATA':
      return (action.data !== state.data)
        ? Object.assign({}, state, { data: action.data })
        : state;

    case 'TREE::DETAILS':
      return Object.assign({}, state, { getDetails: action.getDetails });

    default:
      return state;
  }
}


function instructions(state = { data: '' }, action) {
  const merge = (data) => {
    const all = state.data + data + '\n\n* WAITING FOR USER INPUT *\n';
    let lines = all.split('\n');
    const n = lines.length;

    if (n > 2000) lines = lines.slice(-2000);

    lines = lines.map(l =>
      l.replace(/^([ 0-9a-f]{5,}): (\w+) /, '<i>$1</i>: <b>$2</b>'));

    return lines.join('\n');
  };

  switch (action.type) {
    case 'INSTRUCTIONS':
      return (action.data !== state.data)
        ? { data: merge(action.data) }
        : state;

    default:
      return state;
  }
}


const initialModal = {
  child: null,
};

function modal(state = initialModal, action) {
  switch (action.type) {
    case 'MODAL::SHOW':
      return { child: action.child };

    case 'MODAL::HIDE':
      return { child: null };

    default:
      return state;
  }
}


const initialSettings = {
  map:    !!JSON.parse(localStorage.getItem('setting:map')),
  tree:   !!JSON.parse(localStorage.getItem('setting:tree')),
  plain:  !!JSON.parse(localStorage.getItem('setting:plain')),
  read:   !!JSON.parse(localStorage.getItem('setting:read')),
  listen: !!JSON.parse(localStorage.getItem('setting:listen')),
  instructions: !!JSON.parse(localStorage.getItem('setting:instructions')),
};

function settings(state = initialSettings, action) {
  switch (action.type) {
    case 'SETTING':
      if (state[action.name] === action.value) return state;

      localStorage.setItem(`setting:${action.name}`, JSON.parse(action.value));
      return Object.assign({}, state, { [action.name]: action.value });

    default:
      return state;
  }
}


const initialSaves = {
  current: null,
  saves: [],
};

function saves(state = initialSaves, action) {
  const toObj = ([id, data]) => ({ id, data });

  switch (action.type) {
    case 'SAVES::STATE':
      return Object.assign({}, state, { current: action.save });

    case 'SAVES::LOAD':
      return Object.assign({}, state, {
        saves: action.saves.map(JSON.parse).map(toObj),
      });

    case 'SAVES::INSTR':
      return Object.assign({}, state, {
        saves: [...state.saves, toObj(JSON.parse(action.save))]
      });

    default:
      return state;
  }
}


export default combineReducers({
  transcript,
  map,
  tree,
  instructions,
  settings,
  saves,
  modal,
});
