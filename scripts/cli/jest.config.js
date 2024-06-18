/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: "ts-jest/presets/js-with-babel",
  transformIgnorePatterns: [
    "<rootDir>/node_modules/(?!@polkadot|@babel/runtime/helpers/esm/)",
  ],
  roots: ["<rootDir>/src"],
  modulePaths: [
    "<rootDir>",
		"<rootDir>/src/interfaces/",
  ],
  testRegex: "(/__tests__/.*|\\.(test|spec))\\.(ts|tsx)$",
  moduleFileExtensions: ["ts", "tsx", "js"],
  setupFilesAfterEnv: ["<rootDir>/src/__tests__/setup.ts"],
	testEnvironment: "node",
  testPathIgnorePatterns: ["<rootDir>/src/__tests__/setup.ts"],
  testTimeout: 1_000_000,
	testSequencer: "<rootDir>/src/testSequencer.js",
  moduleNameMapper: {
    '~/(.*)': '<rootDir>/src/$1',
    'polymesh-types/(.*)': '<rootDir>/src/interfaces/$1',
  }
};
