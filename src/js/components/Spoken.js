import React, { Component } from 'react';
import { shallowEqual } from 'shouldcomponentupdate-children';


// non-local voices cut out after > ~15s of speech. need to break them up into
// multiple utterances to prevent this.
const CHARACTER_LIMIT = 200;
let google_uk;


// this is async for some reason:
window.speechSynthesis && (window.speechSynthesis.onvoiceschanged = function() {
  window.speechSynthesis.getVoices().forEach((voice) => {
    if (voice.voiceURI === 'Google UK English Male') google_uk = voice;
  });
});


function makeUtterance(text) {
  const utterance = new SpeechSynthesisUtterance(text);

  // try to make is sound nice & at a reasonable speed
  // - iOS is faster than windows voices
  // - I like the UK google voice if available
  if (/iPad|iPhone|iPod/.test(navigator.userAgent)) {
    utterance.rate = 1;
  } else if (google_uk) {
    utterance.voice = google_uk;
    utterance.pitch = 0.85;
    utterance.rate = 1;
  } else {
    utterance.pitch = 0.9;
    utterance.rate = 1.25;
  }

  return utterance;
}


function segmentize(text) {
  let segments = [['']];
  let count = 0;

  text.split(/(?=,|\.|;)/).forEach((clump) => {
    if (clump.length + count > CHARACTER_LIMIT) {
      count = clump.length;
      segments.push([clump]);
    } else {
      count += clump.length + 1;
      segments[segments.length-1].push(clump);
    }
  });

  // merge clumps into whole strings
  segments = segments.map(seg => seg.join(''));

  // look ahead matching puts the delimiters on the next line,
  // so add them back to the previous so it doesn't say 'dot'
  segments.forEach((str, i) => {
    if (i && str[0] === ',') segments[i-1] += ',';
    if (i && str[0] === '.') segments[i-1] += '.';
    if (i && str[0] === ';') segments[i-1] += ';';

    segments[i] = str.replace(/^[,.;]/, '');
  });

  // now further break them into lines to make sure there are
  // adequate pauses where you would expect them.
  // (the microsoft voice does a better job of this)
  return segments.join('\n').split('\n');
}


class Spoken extends Component {
  constructor(props) {
    super(props);
    this.elements = [];
  }

  speak(text) {
    if (!window.speechSynthesis) return;

    // replace some troublesome characters (> gets pronounced "greater than")
    const parts = text
      .replace(/Oh, no!/g, 'Oh no! \n')
      .replace(/died {2}\*\*/g, 'died. \n')
      .replace(/:/g, '. ')
      .replace(/>|<|\*/g, '')
      .split('\n');

    // break room into its own utterance so it gets a proper pause
    // the rest needs to be broken into ~15 second chunks or else it'll cut off
    const room = makeUtterance(parts.shift());
    const rest = segmentize(parts.join('\n')).map(makeUtterance);

    // cancel any previous
    window.speechSynthesis.cancel();

    // short delay seems necessary to make sure it works
    setTimeout(() => {
      window.speechSynthesis.speak(room);
      rest.forEach(utter => window.speechSynthesis.speak(utter));
    }, 500);
  }

  shouldComponentUpdate(nextProps) {
    return shallowEqual(this.props, nextProps);
  }

  componentDidMount() {
    if (this.props.isEnabled) {
      requestAnimationFrame(() => {
        const text = this.elements.map(el => el.innerText).join('\n');
        this.speak(text);
      });
    }
  }

  render() {
    this.elements = [];

    const clone = child => React.cloneElement(child, {
      ref: (el) => {
        if (el && child.ref) child.ref(el);
        if (el) this.elements.push(el);
      },
    });

    return React.Children.map(this.props.children, clone);
  }
}


export default Spoken;
