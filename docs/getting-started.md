# Getting started with Toast
Toast is an ES module based system for building websites. 

It allows you to use Preact and MDX along with other web technologies you might want to add, but without using a bundler like Webpack or Rollup. 

The end result is that ES modules are served to the browser instead of compiled ES6+ files.

Here are the minimal steps needed to create a Toast site:

```bash
npx create-toast
```

Note: Npx is a tool installed with NPM that allows you to be able to install depdencies without doing `npm install`. It's useful for running run off scripts, like creating a Toast site. 

The centtal premise of Toast all starts with the `esinstall` file, which will be explained below. 

## Example esinstall file
The esinstall file lets us define what packages we want to include in our final build directory. 

Here is an example file:

```javascript
export const specs = [
  "preact",
  "preact/hooks",
  "react-helmet",
  "preact/compat",
];

export const options = {
  alias: {
    react: "preact/compat",
  },
};

```

The file that interacts with the esinstall file is the postinstall.js file.

The first section after the imports creates a logger which we can use when converting our modules to ESM. 

The `main` function does all of the heavy lifting of converting the node_modules to ESM, so we dont need to worry about configuration. 

```javascript
#!/usr/bin/env node
import { install, printStats } from "esinstall";
import prettyBytes from "pretty-bytes";
import cTable from "console.table";
import { options, specs } from "./esinstall.js";

// esinstall doesn't let us quiet the output while it runs
// so we kinda do that here.
const logger = {
  debug() {},
  warn(...args) {
    console.warn(...args);
  },
  error(...args) {
    console.error(...args);
  },
};
async function main() {
  const { success, stats } = await install(specs, {
    dest: "./public/web_modules",
    // logger,
    ...options,
  });
  if (stats) {
    console.table(
      Object.entries(stats.direct)
        .map(([key, value]) => ({
          esm: key,
          ...Object.fromEntries(
            Object.entries(value).map(([k, v]) => [k, prettyBytes(v)])
          ),
        }))
        .concat(
          Object.entries(stats.common).map(([key, value]) => ({
            esm: key,
            ...Object.fromEntries(
              Object.entries(value).map(([k, v]) => [k, prettyBytes(v)])
            ),
          }))
        )
    );
  }
}

try {
  main();
} catch (e) {
  throw e;
}

```


## More Advanced configuration



## Using a Toast file
central to toast is the concept of MDX files(.mdx) which allow you to use React/Preact components within
a Javascript file along with markdown. This a llows you to create things like a component showcase for example
To use MDX in toast we will need to create something called a Toast file.

Create a file called `toast.js` in the root of your project.
Here is an example:

```javascript
import { sourceMdx } from "@toastdotdev/mdx";

export const sourceData = async ({ setDataForSlug }) => {
  await sourceMdx({
    setDataForSlug,
    directory: "./content",
    slugPrefix: "/",
  });
  return;
};

```