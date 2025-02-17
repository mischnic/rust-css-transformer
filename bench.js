const css = require('./native');
const cssnano = require('cssnano');
const postcss = require('postcss');
const esbuild = require('esbuild');

let opts = {
  filename: process.argv[process.argv.length - 1],
  code: require('fs').readFileSync(process.argv[process.argv.length - 1])
};

async function run() {
  await doCssNano();

  console.time('esbuild');
  let r = await esbuild.transform(opts.code.toString(), {
    sourcefile: opts.filename,
    loader: 'css',
    minify: true
  });
  console.timeEnd('esbuild');
  console.log(r.code.length + ' bytes');
  console.log('');

  console.time('parcel-css');
  let res = css.transform(opts);
  console.timeEnd('parcel-css');
  console.log(res.length + ' bytes');
}

async function doCssNano() {
  console.time('cssnano');
  const result = await postcss([
    cssnano,
  ]).process(opts.code, {from: opts.filename});
  console.timeEnd('cssnano');
  console.log(result.css.length + ' bytes');
  console.log('');
}

run();
