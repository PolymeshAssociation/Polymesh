module.exports = {
  // Convert ESM -> CJS; Specifically for @polkadot >= v4 where ESM becomes default
  plugins: ['@babel/plugin-transform-modules-commonjs'],
};
