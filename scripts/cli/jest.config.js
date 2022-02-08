/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
  preset: "ts-jest/presets/js-with-babel",
  transformIgnorePatterns: [
    "<rootDir>/node_modules/(?!@polkadot|@babel/runtime/helpers/esm/)",
  ],
  roots: ["<rootDir>/src"],
  testRegex: "(/__tests__/.*|\\.(test|spec))\\.(ts|tsx)$",
  moduleFileExtensions: ["ts", "tsx", "js"],
  setupFilesAfterEnv: ["<rootDir>/src/__tests__/setup.ts"],
  testPathIgnorePatterns: ["<rootDir>/src/__tests__/setup.ts"],
  testTimeout: 1_000_000,
  moduleNameMapper: {
    'polymesh-typegen/(.*)': '<rootDir>/src/interfaces/$1',
    '@polkadot/api/augment': '<rootDir>/src/interfaces/augment-api.ts',
  }
};
