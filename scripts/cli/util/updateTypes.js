const fs = require("fs");
const path = require("path");
const PrettyError = require("pretty-error");
const pe = new PrettyError();
const filePath = path.join(__dirname + "/../../../polymesh_schema.json");
const { types } = JSON.parse(fs.readFileSync(filePath, "utf8"));
const newTypes = `export default {\ntypes: ${JSON.stringify(types)}\n}`;
const destinationPath = path.join(__dirname + "/../src/interfaces/definitions.ts");
fs.writeFile(
	destinationPath,
	newTypes,
	"utf8",
	(err) => {
		if (err) {
			console.error(pe.render(err));
			process.exit(1);
		}
	}
);
