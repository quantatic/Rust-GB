import path from 'path';
import CopyPlugin from 'copy-webpack-plugin';
import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin';
import {Configuration} from 'webpack';

const dist = path.resolve(__dirname, 'dist');


const config: Configuration = {
  mode: 'production',
  entry: {
    index: './js/index.ts',
  },
  output: {
    path: dist,
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: [
          /node_modules/,
        ],
      },
      {
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
      },
    ],
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.css', '...'],
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        path.resolve(__dirname, 'static')],
    }),
    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
    topLevelAwait: true,
  },
};

export default config;
