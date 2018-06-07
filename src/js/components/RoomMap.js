import React, { Component } from 'react';
import { connect } from 'react-redux';

import { D3Map } from '../Rooms';


class RoomMap extends Component {
  componentDidMount() {
    this.map = null;
    if (this.props.graph) this.make();
  }

  make() {
    this.map = new D3Map(this.el);
    this.map.set(this.props.graph);

    setTimeout(() => {
      this.map.center();
      this.map.update();
    });
  }

  componentDidUpdate() {
    // first map load or after a map clear:
    if (!this.map || this.map.graph !== this.props.graph) this.make();
    else if (this.map && this.props.isVisible) this.map.update();

    if (this.props.centered) this.map.center();
  }

  render() {
    return <div className="rooms" ref={el => this.el = el}></div>;
  }
}


export default connect(
  state => ({
    graph: state.map.graph,
    update: state.map.update, // empty obj flag, just to make D3 update
  })
)(RoomMap);
