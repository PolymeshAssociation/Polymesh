const schema = JSON.parse(require('fs').readFileSync('../../polymesh_schema.json'));

const disallowedTypes = ["u128", "balance"];

function main() {
    for (const module in schema.rpc) {
        for (const rpc in schema.rpc[module]) {
            console.log(`Checking ${rpc}`);
            walk(schema.rpc[module][rpc].type);
        }
    }
}

main();

function walk(returnType) {
    // If it's a custom defined object, fetch the definition
    if (schema.types[returnType]) {
        walk(schema.types[returnType]);
        return;
    }

    if (returnType === Object(returnType)) {
        for (const element in returnType) {
            walk(returnType[element]);
        }
        return;
    }

    // If there's a vector or option, strip it
    returnType = returnType.replace(/(vec)|(option)|[<>\(\) \[\]]/ig, '');
    returnType = returnType.replace(';', ',');

    // If it's a custom defined object after stripping, fetch the definition
    if (schema.types[returnType]) {
        walk(schema.types[returnType]);
        return;
    }

    // If it's a tupple, call walk over all keys
    if (returnType.includes(",")) {
        returnType.split(",").forEach(element => {
            walk(element);
        });
        return;
    }

    // It's a primitive type, ensure it's not disallowed
    if (disallowedTypes.includes(returnType.toLowerCase())) {
        console.log("Error, disallowed types found:", returnType);
        process.exit(1);
    }
}
