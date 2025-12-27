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

function getBundledServerPath(context: ExtensionContext): string | undefined {
  const platform = process.platform;
  const arch = process.arch;

  let binaryName: string;
  if (platform === "win32") {
    binaryName = "bobbin-lsp.exe";
  } else if (platform === "darwin") {
    binaryName = `bobbin-lsp-${arch === "arm64" ? "aarch64" : "x86_64"}-apple-darwin`;
  } else if (platform === "linux") {
    binaryName = `bobbin-lsp-${arch === "arm64" ? "aarch64" : "x86_64"}-unknown-linux-gnu`;
  } else {
    return undefined;
  }

  return path.join(context.extensionPath, "bin", binaryName);
}

function getExecutableName(name: string): string {
  return process.platform === "win32" ? `${name}.exe` : name;
}

function getHomeDir(): string {
  return process.env.HOME || process.env.USERPROFILE || "";
}
