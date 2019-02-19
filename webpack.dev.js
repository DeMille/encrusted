const path = require('path');

const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
  mode: 'development',

  entry: {
    bundle: './src/js/index.js',
    worker: './src/js/worker.js'
  },

  output: {
    filename: '[name].js',
    path: path.join(__dirname, '/build'),
  },

  module: {
    rules: [
      {
        test: /.jsx?$/,
        loader: 'babel-loader',
        exclude: /node_modules/,
        query: {
          presets: ['react']
        }
      }
    ]
  },

  plugins: [
    new CopyWebpackPlugin([
      { from: './src/dev.html', to: './index.html' },
      { from: './src/*.css', to: './[name].[ext]' },
      { from: './src/img/**.*', to: './img/[name].[ext]' },
    ]),
  ],

  devServer: {
    historyApiFallback: {
      rewrites: [
        { from: /^\/run\/.+/, to: '/index.html' },
        { from: /./, to: '/404.html' }
      ]
    }
  }
};
