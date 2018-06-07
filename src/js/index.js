import React from 'react';
import ReactDOM from 'react-dom';
import { createStore, applyMiddleware } from 'redux';
import { Provider } from 'react-redux';
import { BrowserRouter, Switch, Route } from 'react-router-dom';

import ZMachine from './components/ZMachine';
import Launcher from './components/Launcher';

import middleware from './middleware';
import reducer from './reducer';

const store = createStore(reducer, applyMiddleware(middleware));

const basename = (process.env.NODE_ENV === 'production')
  ? '/encrusted'
  : '/';

ReactDOM.render(
  <Provider store={store}>
    <BrowserRouter basename={basename}>
      <Switch>
        <Route exact path="/" component={Launcher} />
        <Route path="/run/:filename" component={ZMachine} />
      </Switch>
    </BrowserRouter>
  </Provider>,
  document.getElementById('root')
);
