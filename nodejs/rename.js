const { renameSync } = require("fs");

const newFileName = [process.platform, process.arch].join("__");
renameSync('index.node.gz', `${newFileName}.node.gz`);
