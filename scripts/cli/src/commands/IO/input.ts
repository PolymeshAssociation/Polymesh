var readlineSync = require('readline-sync');

function readAddress(message: string, defaultValue: string) {
  return readlineSync.question(message, {
    limit: function (input: string) {
      // return web3.utils.isAddress(input);
      return ;
    },
    limitMessage: "Must be a valid address",
    defaultInput: defaultValue
  });
}

function readMultipleAddresses(message: string) {
  return readlineSync.question(message, {
    limit: function (input: string) {
      // return input === '' || input.split(",").every(a => web3.utils.isAddress(a));
      return ;
    },
    limitMessage: `All addresses must be valid`
  });
}

function readNumberGreaterThan(minValue: number, message: string, defaultValue: number) {
  return readlineSync.question(message, {
    limit: function (input: any) {
      return parseFloat(input) > minValue;
    },
    limitMessage: `Must be greater than ${minValue}`,
    defaultInput: defaultValue
  });
}

function readNumberGreaterThanOrEqual(minValue: number, message: string, defaultValue: number) {
  return readlineSync.question(message, {
    limit: function (input: any) {
      return parseFloat(input) >= minValue;
    },
    limitMessage: `Must be greater than or equal ${minValue}`,
    defaultInput: defaultValue
  });
}

function readNumberLessThan(maxValue: number, message: string, defaultValue: number) {
  return readlineSync.question(message, {
    limit: function (input: any) {
      return parseFloat(input) < maxValue;
    },
    limitMessage: `Must be less than ${maxValue}`,
    defaultInput: defaultValue
  });
}

function readNumberLessThanOrEqual(maxValue: number, message: string, defaultValue: number) {
  return readlineSync.question(message, {
    limit: function (input: any) {
      return parseFloat(input) < maxValue;
    },
    limitMessage: `Must be less than or equal ${maxValue}`,
    defaultInput: defaultValue
  });
}
  
function readNumberBetween(minValue: number, maxValue: number, message: string, defaultValue: number) {
  return readlineSync.question(message, {
    limit: function (input: any) {
      return parseFloat(input) >= minValue && parseFloat(input) <= maxValue;
    },
    limitMessage: `Must be betwwen ${minValue} and ${maxValue}`,
    defaultInput: defaultValue
  });
}

function readStringNonEmpty(message: string, defaultValue: string) {
  return readlineSync.question(message, {
    limit: function (input: any) {
      return input.length > 0;
    },
    limitMessage: "Must be a valid string",
    defaultInput: defaultValue
  });
}

function readStringNonEmptyWithMaxBinarySize(maxBinarySize: number, message: string, defaultValue: string) {
  return readlineSync.question(message, {
    limit: function (input: any) {
      return input.length > 0 && Buffer.byteLength(input, 'utf8') < maxBinarySize
    },
    limitMessage: `Must be a valid string with binary size less than ${maxBinarySize}`,
    defaultInput: defaultValue
  });
}

function readDateInTheFuture(message: string, defaultValue:any) {
  const now = Math.floor(Date.now() / 1000);
  return readlineSync.question(message, {
    limit: function (input: any) {
      return parseInt(input) >= now;
    },
    limitMessage: `Must be a future date`,
    defaultInput: defaultValue
  });
}

module.exports = {
  readAddress,
  readMultipleAddresses,
  readNumberGreaterThan,
  readNumberGreaterThanOrEqual,
  readNumberLessThan,
  readNumberLessThanOrEqual,
  readNumberBetween,
  readStringNonEmpty,
  readStringNonEmptyWithMaxBinarySize,
  readDateInTheFuture
}