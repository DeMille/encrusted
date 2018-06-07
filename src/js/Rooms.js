import d3 from 'd3';
import LZString from 'lz-string';

import measure from './measure';


const DIRECTIONS = [
  'north', 'east', 'south', 'west',
  'northeast', 'northwest', 'southeast', 'southwest',
  'up', 'down', 'in', 'out',
];

const DASHED = ['up', 'down', 'in', 'out'];

const X_OFFSET = {
  east:      200,
  west:      -200,
  northeast: 200,
  southeast: 200,
  northwest: -200,
  southwest: -200,
  in:        100,
  out:       -100,
  up:        -300,
  down:      300,
};

const Y_OFFSET = {
  north:     -200,
  south:     200,
  northeast: -200,
  northwest: -200,
  southeast: 200,
  southwest: 200,
  in:        80,
  out:       -80,
  up:        -500,
  down:      500,
};


function parse(input = '') {
  const words = input.toLowerCase().split(' ');
  let command;

  words.some((word) => {
    switch (word) {
      case 'n':
      case 'north':
        command = 'north';
        break;
      case 's':
      case 'south':
        command = 'south';
        break;
      case 'e':
      case 'east':
        command = 'east';
        break;
      case 'w':
      case 'west':
        command = 'west';
        break;
      case 'ne':
      case 'northe':
      case 'northeast':
        command = 'northeast';
        break;
      case 'nw':
      case 'northw':
      case 'northwest':
        command = 'northwest';
        break;
      case 'se':
      case 'southe':
      case 'southeast':
        command = 'southeast';
        break;
      case 'sw':
      case 'southw':
      case 'southwest':
        command = 'southwest';
        break;
      case 'up':
      case 'u':
      case 'climb':
        command = 'up';
        break;
      case 'down':
      case 'd':
        command = 'down';
        break;
      case 'in':
      case 'inside':
      case 'enter':
        command = 'in';
        break;
      case 'out':
      case 'outside':
      case 'exit':
        command = 'out';
        break;
      default:
        command = '';
    }

    return !!command;
  });

  return command || '';
}


class Graph {
  constructor() {
    this.nodes = new Map();
    this.edges = new Map();
    this.current = null;
  }

  clear() {
    this.nodes.clear();
    this.edges.clear();
    this.nodes.set(this.current.id, this.current);
  }

  setCurrent(id) {
    if (this.nodes.has(id)) {
      this.current = this.nodes.get(id);
    }
  }

  isCurrent(id) {
    return this.current && this.current.id === id;
  }

  addNode(id, name, alignment) {
    // need a default for fist node
    let x = (this.current) ? this.current.x : 300;
    let y = (this.current) ? this.current.y : 200;

    if (!!~DIRECTIONS.indexOf(alignment)) {
      x += X_OFFSET[alignment] || 0;
      y += Y_OFFSET[alignment] || 0;
    } else if (alignment) {
      x += 250;
    }

    this.nodes.set(id, { id, x, y, n: name });
  }

  addEdge(src, trg, how) {
    const srcN = this.nodes.get(src);
    const trgN = this.nodes.get(trg);

    if (!srcN || !trgN) {
      throw new Error('Missing src or trg node');
    }

    const id = `${src}->${trg}`;
    const old = this.edges.get(id);

    if (old) {
      if (!~old.how.split(', ').indexOf(how)) {
        old.how += ', ' + how;
      }
    } else {
      this.edges.set(id, {
        id,
        src,
        trg,
        how,
        srcN,
        trgN,
        c: (!!~DASHED.indexOf(how)) ? 'd' : 's'
      });
    }
  }

  moveTo(id, name, how = '') {
    const cmds = how
      .split(/\.|,|and/)
      .map(str => parse(str.trim()))
      .filter(cmd => !!cmd);

    const direction = cmds.pop() || '';

    if (!this.nodes.has(id)) {
      this.addNode(id, name, direction);
    }

    if (
      !!direction &&
      !cmds.length &&
      this.current &&
      how !== 'DIED' &&
      how !== 'UNDO' &&
      how !== 'REDO' &&
      how !== 'restore' &&
      how !== 'y' // affirmative, like after a restart
    ) {
      this.addEdge(this.current.id, id, direction);
    }

    this.setCurrent(id);
  }

  obj() {
    return {
      edges: this.edges,
      nodes: this.nodes,
      current: this.current,
    };
  }

  load(contents) {
    this.clear();
    this.nodes = contents.nodes;
    this.edges = contents.edges;
    this.current = contents.current;
  }

  serialize() {
    return LZString.compressToEncodedURIComponent(JSON.stringify({
      edges: [...this.edges.values()],
      nodes: [...this.nodes.values()],
      current: this.current,
    }));
  }

  static deserialize(data) {
    const graph = new Graph();
    let src;

    if (!data) return graph;

    try {
      src = JSON.parse(LZString.decompressFromEncodedURIComponent(data));
    } catch (err) {
      console.log('Error parsing graph:', err, data);
      return graph;
    }

    graph.current = src.current;
    src.nodes.forEach(n => graph.nodes.set(n.id, n));
    src.edges.forEach(e => graph.addEdge(e.src, e.trg, e.how));

    return graph;
  }
}


class D3Map {
  constructor(element) {
    d3.select(element).html('');

    this._svg = d3.select(element).append('svg')
      .attr('width', '100%')
      .attr('height', '100%');

    this.graph = null;

    const map = this._svg.append('g')
      .attr('class', 'map');

    this._paths = map.append('g')
      .attr('class', 'paths')
      .selectAll('g');

    this._path_labels = map.append('g')
      .attr('class', 'path_labels')
      .selectAll('g');

    this._rooms = map.append('g')
      .attr('class', 'rooms')
      .selectAll('g');

    function zoom() {
      d3.select('.map')
        .attr(
          'transform',
          `translate(${d3.event.translate}) scale(${d3.event.scale})`
        );
    }

    // listen for drag/zoom
    this._zoom = d3.behavior.zoom().scaleExtent([0.3, 1.5]).on('zoom', () => zoom());
    this._svg.call(this._zoom).on('dblclick.zoom', null);
  }

  _drag(d) {
    d.x = Math.round((d.x + d3.event.dx) * 100) / 100;
    d.y = Math.round((d.y + d3.event.dy) * 100) / 100;
    this.update();
  }

  _size(height, width) {
    this._svg.attr('height', height).attr('width', width);
  }

  _height() {
    return this._svg[0][0].height.baseVal.value;
  }

  _width() {
    return this._svg[0][0].width.baseVal.value;
  }

  set(graph) {
    this.graph = graph;
  }

  update() {
    this._paths = this._paths.data([...this.graph.edges.values()]);
    this._path_labels = this._path_labels.data([...this.graph.edges.values()]);
    this._rooms = this._rooms.data([...this.graph.nodes.values()]);

    // update existing paths
    this._paths
      .attr('d', d => `M${d.srcN.x},${d.srcN.y}L${d.trgN.x},${d.trgN.y}`);

    // add new paths
    this._paths.enter()
      .append('path')
      .attr('class', d => d.c)
      .attr('id', d => d.id)
      .attr('d', d => `M${d.srcN.x},${d.srcN.y}L${d.trgN.x},${d.trgN.y}`);

    this._path_labels.enter()
      .append('text')
      .attr('dy', -4)
      .append('textPath')
      .text(d => `${d.how} →`)
      .attr('class', 'path-label')
      .attr('xlink:href', d => `#${d.id}`)
      .attr('startOffset', '50%')
      .attr('text-anchor', 'middle');

    // remove old links
    this._paths.exit().remove();

    // update existing nodes
    this._rooms
      .attr('class', d => (this.graph.current.id === d.id) ? 'active room' : 'room')
      .attr('transform', d => `translate(${d.x},${d.y})`);

    // add new nodes
    // (stopPropagation makes it so ONLY this node gets dragged)
    const rooms = this._rooms.enter()
      .append('g')
      .attr('class', d => (this.graph.current.id === d.id) ? 'active room' : 'room')
      .attr('transform', d => `translate(${d.x},${d.y})`)
      .on('mousedown', () => d3.event.stopPropagation())
      .call(d3.behavior.drag()
        .origin(d => ({ x: d.x, y: d.y }))
        .on('drag', d => this._drag(d)));

    rooms.append('rect');
    rooms.append('text')
      .text(d => d.n)
      .attr('class', 'name')
      .attr('text-anchor', 'middle')
      .attr('y', 5);

    rooms.selectAll('text.name').each(function() {
      const text = this; // text element, not `Map` instance
      const font = '13px Playfair Display italic';
      const width = measure(text.textContent, font) + 20;
      const rect = text.previousElementSibling;

      d3.select(rect)
        .attr('width', width)
        .attr('height', 30)
        .attr('x', -width / 2)
        .attr('y', -15);
    });

    // remove old nodes
    this._rooms.exit().remove();
  }

  center() {
    if (!this.graph || !this.graph.current) return;

    const { x, y } = this.graph.current;

    const x0 = -x + (this._width() / 2);
    const y0 = -y + (this._height() / 2) - 50;

    this._zoom.translate([x0, y0]);
    this._zoom.scale(1);
    this._zoom.event(this._svg);
  }
}


export { Graph, D3Map };
