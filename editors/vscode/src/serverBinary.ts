import * as vscode from "vscode";
import * as https from "https";
import * as fs from "fs";
import * as path from "path";
import { execFile } from "child_process";
import { IncomingMessage } from "http";
import { hostTarget, assetFileName } from "./platformTarget";

const GITHUB_REPO = "sjames/syscribe";
const USER_AGENT = "syscribe-vscode-extension";

/** True if `command` runs and responds to `--version` (i.e. is a usable syscribe binary on PATH). */
function commandWorks(command: string): Promise<boolean> {
  return new Promise((resolve) => {
    execFile(command, ["--version"], { timeout: 5000 }, (error) => {
      resolve(!error);
    });
  });
}

const REQUEST_TIMEOUT_MS = 10000;

/**
 * GET a URL following redirects, returning the final response (caller must
 * consume/pipe it). Guards against a stalled connect/response with an
 * explicit timeout — a hung network call must fail, not hang `activate()`.
 */
function httpGet(url: string, headers: Record<string, string> = {}, redirectsLeft = 5): Promise<IncomingMessage> {
  return new Promise((resolve, reject) => {
    const req = https.get(
      url,
      { headers: { "User-Agent": USER_AGENT, ...headers }, timeout: REQUEST_TIMEOUT_MS },
      (res) => {
        const status = res.statusCode ?? 0;
        if (status >= 300 && status < 400 && res.headers.location) {
          res.resume();
          if (redirectsLeft <= 0) {
            reject(new Error(`too many redirects fetching ${url}`));
            return;
          }
          httpGet(res.headers.location, headers, redirectsLeft - 1).then(resolve, reject);
          return;
        }
        if (status < 200 || status >= 300) {
          res.resume();
          reject(new Error(`GET ${url} -> HTTP ${status}`));
          return;
        }
        resolve(res);
      },
    );
    req.on("error", reject);
    req.on("timeout", () => req.destroy(new Error(`GET ${url} timed out after ${REQUEST_TIMEOUT_MS}ms`)));
  });
}

async function httpGetJson<T>(url: string): Promise<T> {
  const res = await httpGet(url, { Accept: "application/vnd.github+json" });
  const chunks: Buffer[] = [];
  for await (const chunk of res) chunks.push(chunk as Buffer);
  return JSON.parse(Buffer.concat(chunks).toString("utf8")) as T;
}

interface GithubRelease {
  tag_name: string;
}

/** Resolve `"latest"` (or a pinned `vX.Y.Z`) to a concrete release tag via the GitHub API. */
async function resolveTag(version: string): Promise<string> {
  if (version !== "latest") return version;
  const release = await httpGetJson<GithubRelease>(
    `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
  );
  return release.tag_name;
}

async function downloadAsset(
  tag: string,
  fileName: string,
  destPath: string,
  progress?: vscode.Progress<{ message?: string }>,
): Promise<void> {
  const url = `https://github.com/${GITHUB_REPO}/releases/download/${tag}/${fileName}`;
  progress?.report({ message: `Downloading ${fileName}...` });
  const res = await httpGet(url);

  await fs.promises.mkdir(path.dirname(destPath), { recursive: true });
  const tmpPath = `${destPath}.download-${process.pid}`;
  await new Promise<void>((resolve, reject) => {
    const out = fs.createWriteStream(tmpPath, { mode: 0o755 });
    res.pipe(out);
    res.on("error", reject);
    out.on("error", reject);
    out.on("finish", resolve);
  });
  await fs.promises.rename(tmpPath, destPath);
  if (process.platform !== "win32") {
    await fs.promises.chmod(destPath, 0o755);
  }
}

/**
 * Resolve the `syscribe` binary to launch as the LSP server.
 *
 * Precedence: explicit `syscribe.serverPath` setting > `syscribe` on PATH >
 * a managed binary downloaded from the `sjames/syscribe` GitHub releases
 * (cached under the extension's global storage, keyed by resolved tag +
 * host target so re-activation is a cache hit, not a re-download).
 */
export async function resolveServerCommand(
  context: vscode.ExtensionContext,
  log: vscode.OutputChannel,
): Promise<string> {
  const config = vscode.workspace.getConfiguration("syscribe");
  const explicitPath = config.get<string>("serverPath", "").trim();
  if (explicitPath.length > 0) {
    log.appendLine(`Using configured syscribe.serverPath: ${explicitPath}`);
    return explicitPath;
  }

  if (await commandWorks("syscribe")) {
    log.appendLine("Found syscribe on PATH.");
    return "syscribe";
  }

  const host = hostTarget(process.platform, process.arch);
  if (!host) {
    throw new Error(
      `No syscribe binary on PATH, and no prebuilt release exists for this platform ` +
        `(${process.platform}/${process.arch}). Install syscribe yourself and set "syscribe.serverPath".`,
    );
  }

  const version = config.get<string>("version", "latest").trim() || "latest";
  const fileName = assetFileName(host.target, host.exe);
  const binDir = path.join(context.globalStorageUri.fsPath, "bin");

  // Pinned version: pure cache hit, no network at all once downloaded once.
  if (version !== "latest") {
    const pinnedPath = path.join(binDir, version, fileName);
    if (fs.existsSync(pinnedPath)) {
      log.appendLine(`Using cached managed binary: ${pinnedPath}`);
      return pinnedPath;
    }
  }

  return vscode.window.withProgress(
    { location: vscode.ProgressLocation.Notification, title: "Syscribe" },
    async (progress) => {
      let tag: string;
      try {
        progress.report({ message: "Resolving syscribe release..." });
        tag = await resolveTag(version);
      } catch (err) {
        // Offline / rate-limited: fall back to whatever's newest already cached, if any.
        const cached = latestCachedBinary(binDir, fileName);
        if (cached) {
          log.appendLine(`Release lookup failed (${String(err)}); using cached binary ${cached}`);
          return cached;
        }
        throw new Error(`Could not resolve a syscribe release to download: ${String(err)}`);
      }

      const destPath = path.join(binDir, tag, fileName);
      if (fs.existsSync(destPath)) {
        log.appendLine(`Using cached managed binary: ${destPath}`);
        return destPath;
      }

      log.appendLine(`Downloading syscribe ${tag} for ${host.target}...`);
      await downloadAsset(tag, fileName, destPath, progress);
      log.appendLine(`Downloaded syscribe ${tag} to ${destPath}`);
      return destPath;
    },
  );
}

/** Parse a `vX.Y.Z` release tag into a tuple for numeric comparison; undefined if it doesn't match. */
function parseSemverTag(tag: string): [number, number, number] | undefined {
  const m = /^v(\d+)\.(\d+)\.(\d+)$/.exec(tag);
  if (!m) return undefined;
  return [Number(m[1]), Number(m[2]), Number(m[3])];
}

function compareTags(a: string, b: string): number {
  const pa = parseSemverTag(a);
  const pb = parseSemverTag(b);
  if (pa && pb) {
    for (let i = 0; i < 3; i++) {
      if (pa[i] !== pb[i]) return pa[i] - pb[i];
    }
    return 0;
  }
  // Not both vX.Y.Z (e.g. a moving "v0" major tag, or a future tag scheme) — fall back
  // to lexical order rather than guessing.
  return a < b ? -1 : a > b ? 1 : 0;
}

/** Newest cached managed binary across all previously-downloaded tags, if any. */
function latestCachedBinary(binDir: string, fileName: string): string | undefined {
  if (!fs.existsSync(binDir)) return undefined;
  const tags = fs
    .readdirSync(binDir, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .filter((tag) => fs.existsSync(path.join(binDir, tag, fileName)))
    .sort(compareTags);
  const newest = tags.at(-1);
  return newest ? path.join(binDir, newest, fileName) : undefined;
}
