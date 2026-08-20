/**
 * Host-platform -> release-target mapping. Deliberately dependency-free (no
 * `vscode` import) so it can be unit-tested with plain mocha/node instead of
 * needing the Extension Development Host — see `src/test/unit/`.
 *
 * Must stay in sync with the `matrix.include` targets `.github/workflows/release.yml`
 * cross-builds and uploads as release assets named `syscribe-<target>[.exe]`.
 */

export interface HostTarget {
  target: string;
  exe: boolean;
}

export function hostTarget(platform: string, arch: string): HostTarget | undefined {
  if (platform === "linux" && arch === "x64") return { target: "x86_64-unknown-linux-gnu", exe: false };
  if (platform === "linux" && arch === "arm64") return { target: "aarch64-unknown-linux-gnu", exe: false };
  if (platform === "darwin" && arch === "x64") return { target: "x86_64-apple-darwin", exe: false };
  if (platform === "darwin" && arch === "arm64") return { target: "aarch64-apple-darwin", exe: false };
  if (platform === "win32" && arch === "x64") return { target: "x86_64-pc-windows-msvc", exe: true };
  return undefined;
}

export function assetFileName(target: string, exe: boolean): string {
  return `syscribe-${target}${exe ? ".exe" : ""}`;
}
