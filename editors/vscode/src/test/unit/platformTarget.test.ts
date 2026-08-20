import * as assert from "assert";
import { hostTarget, assetFileName } from "../../platformTarget";

// Kept in sync by hand with `.github/workflows/release.yml`'s build matrix
// (`matrix.include[].target`) — every asset it uploads must be reachable
// from some (platform, arch) pair below, and vice versa.
const RELEASED_TARGETS = [
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
];

suite("platformTarget", () => {
  test("maps every supported (platform, arch) pair to a released target", () => {
    const cases: Array<[string, string, string, boolean]> = [
      ["linux", "x64", "x86_64-unknown-linux-gnu", false],
      ["linux", "arm64", "aarch64-unknown-linux-gnu", false],
      ["darwin", "x64", "x86_64-apple-darwin", false],
      ["darwin", "arm64", "aarch64-apple-darwin", false],
      ["win32", "x64", "x86_64-pc-windows-msvc", true],
    ];
    for (const [platform, arch, expectedTarget, expectedExe] of cases) {
      const result = hostTarget(platform, arch);
      assert.ok(result, `expected a target for ${platform}/${arch}`);
      assert.strictEqual(result?.target, expectedTarget);
      assert.strictEqual(result?.exe, expectedExe);
      assert.ok(
        RELEASED_TARGETS.includes(result!.target),
        `${result!.target} is not in the release.yml build matrix`,
      );
    }
  });

  test("returns undefined for unsupported platforms", () => {
    assert.strictEqual(hostTarget("linux", "ia32"), undefined);
    assert.strictEqual(hostTarget("win32", "arm64"), undefined);
    assert.strictEqual(hostTarget("freebsd", "x64"), undefined);
  });

  test("assetFileName matches release.yml's staged binary names", () => {
    assert.strictEqual(assetFileName("x86_64-unknown-linux-gnu", false), "syscribe-x86_64-unknown-linux-gnu");
    assert.strictEqual(
      assetFileName("x86_64-pc-windows-msvc", true),
      "syscribe-x86_64-pc-windows-msvc.exe",
    );
  });
});
