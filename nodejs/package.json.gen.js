const fs = require("fs");

const common = {
  version: "0.8.5",
  main: "index.js",
  types: "index.d.ts",
  files: ["postinstall.js", "index.d.ts", "index.js", "binaries"],
  scripts: {
    build: "node-gyp build && shx mv build/Release/index.node index.node",
    clean:
      "cd native && cargo clean && cd .. && node-gyp clean && node-gyp configure && shx rm -f index.node",
    postinstall: "node postinstall.js",
  },
  repository: {
    type: "git",
    url: "git+https://github.com/wilsonzlin/minify-html.git",
  },
  author: {
    email: "npm@wilsonl.in",
    name: "Wilson Lin",
    url: "https://wilsonl.in/",
  },
  license: "MIT",
  bugs: {
    url: "https://github.com/wilsonzlin/minify-html/issues",
  },
  engines: {
    node: ">= 8.6.0",
  },
  homepage: "https://github.com/wilsonzlin/minify-html#readme",
  devDependencies: {
    "@types/node": "^14.6.0",
    "node-gyp": "^7.0.0",
    shx: "^0.3.2",
  },
  keywords: ["compress", "compressor", "fast", "html", "minifier", "minify"],
};

const specifics = {
  core: {
    name: "@min-html/core",
    description: "Extremely fast and smart HTML minifier"
  },
  js: {
    name: "@min-html/js",
    description: "Extremely fast and smart HTML + JS + CSS minifier"
  },
}[process.argv[2]];

fs.writeFileSync(
  "package.json",
  JSON.stringify(
    {
      ...common,
      ...specifics,
    },
    null,
    2
  )
);
