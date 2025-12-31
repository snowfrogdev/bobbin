import * as path from "path";
import * as fs from "fs";
import {
  ExtensionContext,
  workspace,
  window,
  commands,
  OutputChannel,
} from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
let outputChannel: OutputChannel;

export async function activate(context: ExtensionContext): Promise<void> {
  outputChannel = window.createOutputChannel("Bobbin Language Server");
  context.subscriptions.push(outputChannel);

  // Find the LSP server executable
  const serverPath = await findServerPath(context);
  if (!serverPath) {
    window.showErrorMessage(
      "Bobbin: Could not find bobbin-lsp executable. " +
        "Install with: cargo install --path lsp"
    );
    return;
  }

  outputChannel.appendLine(`Using language server: ${serverPath}`);

  // Configure server options
  const run: Executable = {
    command: serverPath,
    options: { env: { ...process.env } },
  };
  const debug: Executable = {
    ...run,
    options: { ...run.options, env: { ...process.env, RUST_BACKTRACE: "1" } },
  };
  const serverOptions: ServerOptions = { run, debug };

  // Configure client options
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "bobbin" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.bobbin"),
    },
    outputChannel,
    traceOutputChannel: outputChannel,
  };

  // Create and start the client
  client = new LanguageClient(
    "bobbin-lsp",
    "Bobbin Language Server",
    serverOptions,
    clientOptions
  );

  // Register restart command
  context.subscriptions.push(
    commands.registerCommand("bobbin.restartServer", async () => {
      if (client) {
        await client.restart();
        window.showInformationMessage("Bobbin language server restarted");
      }
    })
  );

  // Start the client
  await client.start();
  outputChannel.appendLine("Bobbin language server started");
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
  }
}

async function findServerPath(
  context: ExtensionContext
): Promise<string | undefined> {
  // 1. Check user configuration
  const config = workspace.getConfiguration("bobbin");
  const configPath = config.get<string>("lsp.path");
  if (configPath && fs.existsSync(configPath)) {
    return configPath;
  }

  // 2. Check bundled binary in extension
  const bundledPath = getBundledServerPath(context);
  if (bundledPath && fs.existsSync(bundledPath)) {
    return bundledPath;
  }

  // 3. Check cargo install location
  const cargoHome = process.env.CARGO_HOME || path.join(getHomeDir(), ".cargo");
  const cargoPath = path.join(
    cargoHome,
    "bin",
    getExecutableName("bobbin-lsp")
  );
  if (fs.existsSync(cargoPath)) {
    return cargoPath;
  }

  // 4. Check if available in PATH (let the system find it)
  return "bobbin-lsp";
}

// Platform-specific binary names (matches vsce target names)
const PLATFORM_BINARIES: Record<string, string | undefined> = {
  "win32-x64": "bobbin-lsp-win32-x64.exe",
  "darwin-x64": "bobbin-lsp-darwin-x64",
  "darwin-arm64": "bobbin-lsp-darwin-arm64",
  "linux-x64": "bobbin-lsp-linux-x64",
  "linux-arm64": "bobbin-lsp-linux-arm64",
};

function getBundledServerPath(context: ExtensionContext): string | undefined {
  const key = `${process.platform}-${process.arch}`;
  const binaryName = PLATFORM_BINARIES[key];
  return binaryName ? path.join(context.extensionPath, "bin", binaryName) : undefined;
}

function getExecutableName(name: string): string {
  return process.platform === "win32" ? `${name}.exe` : name;
}

function getHomeDir(): string {
  return process.env.HOME || process.env.USERPROFILE || "";
}
