function measure(text, font) {
  const canvas = measure._canvas || document.createElement('canvas');
  if (!measure._canvas) measure._canvas = canvas;

  const context = canvas.getContext('2d');
  context.font = font;

  return context.measureText(text).width;
}

export default measure;
