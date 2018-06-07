import React, { Component } from 'react';
import { connect } from 'react-redux';


class Instructions extends Component {
  componentDidMount() {
    this.el && requestAnimationFrame(() => {
      this.el.scrollTop = this.el.scrollHeight;
    });
  }

  componentDidUpdate() {
    this.el && requestAnimationFrame(() => {
      this.el.scrollTop = this.el.scrollHeight;
    });
  }

  render() {
    return (
      <div className="instructions">
        <div className="wrapper" ref={el => this.el = el}>
          <pre dangerouslySetInnerHTML={{ __html: this.props.instructions }}></pre>
        </div>
      </div>
    );
  }
}


export default connect(
  state => ({
    instructions: state.instructions.data,
  })
)(Instructions);
