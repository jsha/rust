const searchIndex = require('../../../build/x86_64-unknown-linux-gnu/doc/search-index1.59.0.js').searchIndex;
const searchJS = require('../../../build/x86_64-unknown-linux-gnu/doc/search1.59.0.js');
const searchWords = searchJS.initSearch(searchIndex);
results = searchJS.execQuery(searchJS.getQuery("string"), searchWords, undefined, 'std');

let i = 0;
results.others.forEach((r) => {
  let path = r.path;
  if (r.parent) {
    path += `::${r.parent.name}`;
  }
  path += `::${r.name}`;
  console.log(`${i++} : ${path}`);
});
