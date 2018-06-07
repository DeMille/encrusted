
import React, { Component } from 'react';
import { connect } from 'react-redux';

import Tree from '../Tree';

class ObjectTree extends Component {
  componentDidMount() {
    this.tree = new Tree(this.el);
    this.tree.getDetails = this.props.getDetails;
    this.tree.showDetails = this.props.showDetails;

    if (this.props.data && this.props.isVisible) {
      setTimeout(() => this.tree.update(JSON.parse(this.props.data)));
    }
  }

  componentDidUpdate() {
    if (this.props.isVisible) {
      this.tree.update(JSON.parse(this.props.data));
      this.tree.getDetails = this.props.getDetails;
      this.tree.showDetails = this.props.showDetails;
    }
  }

  render() {
    return (
      <div className="tree" ref={el => this.el = el}>
        <div id="tooltip" className="hidden"></div>
      </div>
    );
  }
}

export default connect(
  state => ({
    data: state.tree.data,
    getDetails: state.tree.getDetails,
  }),
)(ObjectTree);
