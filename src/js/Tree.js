import d3 from 'd3';
import Popper from 'popper.js';


const TRANSITION_DURATION = 500;


class Tree {
  constructor(element) {
    this._el = element;

    this.getDetails = () => {};
    this.showDetails = false;
    this._tooltipTimer = null;

    const zoom = () => {
      this._group.attr(
        'transform',
        `translate(${d3.event.translate}) scale(${d3.event.scale})`
      );
    };

    this._zoom = d3.behavior.zoom()
      .scaleExtent([0.3, 1.3])
      .on('zoom', zoom);

    this._svg = d3.select(element).append('svg')
      .attr('width', '100%')
      .attr('height', '100%')
      .call(this._zoom)
      .on('dblclick.zoom', null);

    this._group = this._svg.append('g');
    this._diagonal = d3.svg.diagonal().projection(d => [d.y, d.x]);

    this._yourname = '';
    this._root = {};
    this._root.x0 = this._height() / 2;
    this._root.y0 = this._width() / 2;

    this._tree = d3.layout.tree()
      .size([this._height(), this._width()]);

    this.sort();
  }

  sort() {
    this._tree.sort((a, b) => {
      if (b.name < a.name) return 1;
      if (b.name > a.name) return -1;
      if (b.number < a.number) return 1;
      if (b.number > a.number) return -1;
      return 0;
    });
  }

  _height() {
    return this._svg[0][0].height.baseVal.value;
  }

  _width() {
    return this._svg[0][0].width.baseVal.value;
  }

  _centerNode(node) {
    const scale = this._zoom.scale();
    const x = -node.y0 * scale + this._width() / 2;
    const y = -node.x0 * scale + this._height() / 2;

    this._group.transition()
      .duration(TRANSITION_DURATION)
      .attr('transform', `translate(${x},${y}) scale(${scale})`);

    this._zoom.scale(scale);
    this._zoom.translate([x, y]);
  }

  close(node) {
    node._children = node.children;
    node.children = null;
  }

  open(node) {
    node.children = node._children;
    node._children = null;
  }

  toggle(node) {
    (node.children)
      ? this.close(node)
      : this.open(node);
  }

  _click(node) {
    this.toggle(node);
    this._renderNode(node);
  }

  _showDetails(node) {
    const el = document.querySelector(`#n${node.number}`);
    const tooltip = document.querySelector('#tooltip');

    this.getDetails(node.number).then((details) => {
      const listener = () => {
        tooltip.innerHTML = '';
        tooltip.classList.add('hidden');
        tooltip.removeEventListener('mouseleave', listener);
      };

      tooltip.innerHTML = `
        <h4>Object #${node.number}</h4>
        <span class="close">
          <i
            class="icon ion-ios-close-empty"
            onclick="this.parentElement.parentElement.classList.add('hidden')"
          >
          </i>
        </span>
        <pre class="mb-0">${details}</pre>
      `;
      tooltip.classList.remove('hidden');
      tooltip.addEventListener('mouseleave', listener);

      new Popper(el, tooltip, { placement: 'top-start' });
    });
  }

  _context(node) {
    d3.event.preventDefault();
    this._showDetails(node);
  }

  _mouseover(node) {
    if (this.showDetails) this._showDetails(node);
  }

  _mouseleave() {
    if (!this.showDetails) return;

    const tooltip = document.querySelector('#tooltip');

    if (!tooltip.matches(':hover')) {
      tooltip.innerHTML = '';
      tooltip.classList.add('hidden');
    }
  }

  _maxHeight(max, node) {
    if (!node.children) return max;

    // get height of children
    const height = node.children.reduce((h, child) => {
      if (child.children) h += child.children.length;
      return h;
    }, 0);

    // update the current biggest hieght in the tree
    max = d3.max([max, height]);

    return node.children.reduce(this._maxHeight.bind(this), max);
  }

  _renderNode(src) {
    // 40 units per line
    const height = this._maxHeight(1, this._root) * 40;
    this._tree = this._tree.size([height, this._width()]);

    const nodes = this._tree.nodes(this._root);
    const links = this._tree.links(nodes);

    // 200 width per level
    nodes.forEach((d) => { d.y = (d.depth * 200); });

    // update existing nodes
    const node = this._group.selectAll('g.node').data(nodes, d => d.number);

    const newNode = node.enter().append('g')
      .attr('class', d => (d.name === this._yourname) ? 'you node' : 'node')
      .attr('transform', () => `translate(${src.y0},${src.x0})`)
      .attr('id', d => `n${d.number}`)
      .on('click', this._click.bind(this))
      .on('contextmenu', this._context.bind(this))
      .on('mouseover', this._mouseover.bind(this))
      .on('mouseleave', this._mouseleave.bind(this));

    newNode.append('circle')
      .attr('class', d => (d._children) ? 'nodeCircle collapsed' : 'nodeCircle');

    newNode.append('text')
      .text(d => (d.name === 'cretin') ? 'cretin (you)' : d.name)
      .attr('class', 'text-bg')
      .attr('dy', '.25em')
      .attr('x', d => (d.children || d._children) ? -10 : 10)
      .attr('text-anchor', d => (d.children || d._children) ? 'end' : 'start')
      .style('fill-opacity', 0);

    newNode.append('text')
      .text(d => (d.name === 'cretin') ? 'cretin (you)' : d.name)
      .attr('class', 'object-name')
      .attr('dy', '.25em')
      .attr('x', d => (d.children || d._children) ? -10 : 10)
      .attr('text-anchor', d => (d.children || d._children) ? 'end' : 'start')
      .style('fill-opacity', 0);

    node.selectAll('text')
      .text(d => (d.name === 'cretin') ? 'cretin (you)' : d.name)
      .attr('x', d => (d.children || d._children) ? -10 : 10)
      .attr('text-anchor', d => (d.children || d._children) ? 'end' : 'start');

    node.select('circle.nodeCircle')
      .attr('class', d => (d._children) ? 'nodeCircle collapsed' : 'nodeCircle')
      .attr('r', 4.5);

    // transition nodes to their new position
    const nodeUpdate = node.transition()
      .duration(TRANSITION_DURATION)
      .attr('transform', d => `translate(${d.y},${d.x})`);

    // fade text in
    nodeUpdate.selectAll('text')
      .style('fill-opacity', 1)
      .style('stroke-opacity', 1);

    // transition exiting nodes
    const nodeExit = node.exit();

    nodeExit.transition()
      .duration(TRANSITION_DURATION)
      .attr('transform', () => `translate(${src.y},${src.x})`)
      .remove();

    nodeExit.selectAll('text')
      .transition()
      .duration(TRANSITION_DURATION)
      .style('fill-opacity', 0)
      .style('stroke-opacity', 0);

    nodeExit.selectAll('circle')
      .transition()
      .duration(TRANSITION_DURATION)
      .attr('r', 0);

    // epdate existing links
    const link = this._group.selectAll('path.link')
      .data(links, d => d.target.number);

    // Enter any new links at the parent's previous position.
    link.enter().insert('path', 'g')
      .attr('class', 'link')
      .attr('d', () => this._diagonal({
        source: { x: src.x0, y: src.y0 },
        target: { x: src.x0, y: src.y0 },
      }));

    // Transition links to their new position.
    link.transition()
      .duration(TRANSITION_DURATION)
      .attr('d', this._diagonal);

    // Transition exiting nodes to the parent's new position.
    link.exit().transition()
      .duration(TRANSITION_DURATION)
      .attr('d', () => this._diagonal({
        source: { x: src.x, y: src.y },
        target: { x: src.x, y: src.y },
      }))
      .remove();

    // Stash the old positions for transition.
    nodes.forEach((d) => {
      d.x0 = d.x;
      d.y0 = d.y;
    });
  }

  find(value, prop = 'name') {
    let found;

    function find(node) {
      if (found) return;

      node.children && node.children.forEach((child) => {
        if (child[prop] === value) found = child;
        else find(child);
      });

      node._children && node._children.forEach((child) => {
        if (child[prop] === value) found = child;
        else find(child);
      });
    }

    if (this._root[prop] === value) found = this._root;
    else find(this._root);

    return found;
  }

  you() {
    let you;

    if (this._yourname) {
      you = this.find(this._yourname);
    } else {
      you = this.find('cretin') || this.find('you') || this.find('yourself');
    }

    if (you) this._yourname = you.name;

    return you;
  }

  center(node) {
    this._centerNode(node || this._root);
  }

  render() {
    this._renderNode(this._root);
  }

  update(data) {
    this._root = data;
    this._root.x0 = this._height() / 2;
    this._root.y0 = this._width() / 2;

    const you = this.you();
    this.render();
    this.center(you);
  }
}


export default Tree;
