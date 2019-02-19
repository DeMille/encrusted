const path = require('path');

const webpack = require('webpack');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const TerserPlugin = require('terser-webpack-plugin');

module.exports = {
  mode: 'production',
  devtool: 'source-map',

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
          presets: ['@babel/preset-react'],
        }
      },
    ],
  },

  optimization: {
    minimizer: [
      new TerserPlugin({
        exclude: /\.min\.js$/gi,
        sourceMap: true,
        parallel: true,
      }),
    ],
  },

  plugins: [
    new webpack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production')
    }),

    new webpack.optimize.ModuleConcatenationPlugin(),

    new CopyWebpackPlugin([
      { from: './src/*.html', to: './[name].[ext]' },
      { from: './src/*.css', to: './[name].[ext]' },
      { from: './src/img/**.*', to: './img/[name].[ext]' },
    ]),
  ]
};
