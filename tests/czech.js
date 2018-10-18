const fs = require('fs');
const child = require('child_process');

runTests();

function test(cmd) {
  return new Promise((resolve, reject) => {
    child.exec(cmd, (err, stdout, stderr) => {
      if (err) return reject(stderr);
      resolve(stdout);
    });
  });
}

const ok  = msg => `\x1B[92m${msg}\x1B[0m`;
const err = msg => `\x1B[91m${msg}\x1B[0m`;

async function runTests() {
  const cmd = 'cargo run --bin encrusted -- ./tests/build/czech.z';
  const exp = './tests/expected/expected.';
  const versions = [3, 4, 5, 8];
  const failures = [];

  for (const v of versions) {
    console.log(`\n\x1B[97mTesting command:\x1B[0m ${cmd}${v}`);

    let out = await test(`${cmd}${v}`);
    let expected = fs.readFileSync(`${exp}z${v}.txt`, 'utf-8');

    if (out === expected) {
      console.log(ok('ok.'));
      continue;
    }

    console.log(err('failed.'));
    failures.push([v, stdout]);
  }

  failures.forEach(([v, stdout]) => {
    console.log(`\n\nErrors with version ${v}: \n\n`);
    console.log(stdout);
  });

  if (failures.length) {
    throw new Error(`Failed some tests on ${failures.length} versions`);
  }
}
