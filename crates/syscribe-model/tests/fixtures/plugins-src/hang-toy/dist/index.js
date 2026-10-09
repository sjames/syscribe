// src/index.ts
function parse() {
  let x = 0;
  while (true) {
    x = (x + 1) % 2147483647;
  }
}
module.exports = { parse };
