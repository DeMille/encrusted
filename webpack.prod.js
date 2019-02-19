const path = require('path');

const webpack = require('webpack');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const UglifyPlugin = require('uglifyjs-webpack-plugin');

module.exports = {
  mode: 'production',

  entry: {
    bundle: './src/js/index.js',
    worker: './src/js/worker.js',
  },

  output: {
    filename: '[name].js',
    sourceMapFilename: '[name].map',
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
      },
    ],
  },

  devtool: 'source-map',

  plugins: [
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production')
    }),

    new UglifyPlugin({
      parallel: 4,
      exclude: /\.min\.js$/gi,
      sourceMap: true,
      uglifyOptions: {
        ecma: 6,
        topLevel: true,
        output: {
          comments: false,
        },
        compress: {
          unsafe: true,
        },
      },
    }),

    new webpack.optimize.ModuleConcatenationPlugin(),

    new CopyWebpackPlugin([
      { from: './src/*.html', to: './[name].[ext]' },
      { from: './src/*.css', to: './[name].[ext]' },
      { from: './src/img/**.*', to: './img/[name].[ext]' },
    ]),
  ]
};
