import React, { Component } from 'react';
import { connect } from 'react-redux';
import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';

import Settings from './Settings';
import Help from './Help';
import RoomMap from './RoomMap';
import ObjectTree from './ObjectTree';
import Instructions from './Instructions';


class DebugPanel extends Component {
  constructor() {
    super();

    this.state = {
      tabIndex: 0,
      centered: false,
      showTreeDetails: false,
    };

    this.toggleTreeDetails = this.toggleTreeDetails.bind(this);
    this.centerMap = this.centerMap.bind(this);
  }

  centerMap() {
    this.setState({ centered: true });
    setTimeout(() => this.setState({ centered: false }));
  }

  toggleTreeDetails() {
    this.setState({ showTreeDetails: !this.state.showTreeDetails });
  }

  render() {
    const map = this.props.showMap;
    const tree = this.props.showTree;
    const log = this.props.showInstructions;

    const tabs = [map, tree, log].map((v, id) => ({ isEnabled: v, id }));
    const enabled = tabs.filter(t => !!t.isEnabled);

    // force activate tabs if needed
    let selected = this.state.tabIndex;

    if (!tabs[selected].isEnabled && enabled.length > 0) {
      selected = enabled[0].id;
    }

    // only push buttons to vertical if there are panels open
    const sidebarClass = (enabled.length < 1) ? 'sidebar horizontal' : 'sidebar';

    return (
      <Tabs
        className="debug-panel"
        forceRenderTabPanel
        selectedIndex={selected}
        onSelect={tabIndex => this.setState({ tabIndex })}
      >
        <TabList className={sidebarClass} >

          <li onClick={() => this.props.openModal(<Settings />)}>
            <i className="icon ion-android-menu mb-0"></i>
          </li>
          <li onClick={() => this.props.openModal(<Help />)}>
            <i className="icon ion-asterisk mt-0 hide-sm" style={{ fontSize: '19px' }}></i>
          </li>

          <Tab disabled={!map}>
            <i className="icon ion-location"></i>
          </Tab>
          <Tab disabled={!tree}>
            <i className="icon ion-merge ml-1"></i>
          </Tab>
          <Tab disabled={!log}>
            <i className="icon ion-ios-pulse-strong"></i>
          </Tab>

          {map && selected === 0 &&
            <span className="bottom">
              <li onClick={this.centerMap}>
                <i className="icon ion-pinpoint"></i>
                <span className="text">re-center map</span>
              </li>
            </span>
          }

          {tree && selected === 1 &&
            <span className="bottom">
              <li onClick={this.toggleTreeDetails}>
                {!this.state.showTreeDetails &&
                <i className="icon ion-ios-glasses-outline"></i>
                }
                {this.state.showTreeDetails &&
                  <i className="icon ion-ios-glasses"></i>
                }
                <span className="text">toggle detailed mode</span>
              </li>
            </span>
          }
        </TabList>

        <div className="panels">
          <TabPanel>
            {map &&
              <RoomMap
                isVisible={selected === 0}
                centered={this.state.centered}
              />
            }
          </TabPanel>

          <TabPanel>
            {tree &&
              <ObjectTree
                isVisible={selected === 1}
                showDetails={this.state.showTreeDetails}
              />
            }
          </TabPanel>

          <TabPanel>
            {log && selected === 2 && <Instructions />}
          </TabPanel>
        </div>
      </Tabs>
    );
  }
}


export default connect(
  state => ({
    showMap: state.settings.map,
    showTree: state.settings.tree,
    showInstructions: state.settings.instructions,
  }),
  dispatch => ({
    openModal: child => dispatch({ type: 'MODAL::SHOW', child }),
  }),
)(DebugPanel);
