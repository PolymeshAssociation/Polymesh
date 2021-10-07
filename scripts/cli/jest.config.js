/** @type {import('ts-jest/dist/types').InitialOptionsTsJest} */
module.exports = {
	roots: ["<rootDir>/src"],
	transform: {
		".(ts|tsx)": "ts-jest",
	},
	testRegex: "(/__tests__/.*|\\.(test|spec))\\.(ts|tsx)$",
	moduleFileExtensions: ["ts", "tsx", "js"],
};
