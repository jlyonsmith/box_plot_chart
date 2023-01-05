#!/usr/bin/env -S deno run --unstable --allow-run --allow-read --allow-write

import * as path from "https://deno.land/std@0.167.0/path/mod.ts";
import * as colors from "https://deno.land/std@0.167.0/fmt/colors.ts";
import { parse } from "https://deno.land/std@0.167.0/flags/mod.ts";
import { File } from "https://deno.land/x/fs_pro@3.11.0/mod.ts";
import { pascalCase, snakeCase } from "https://deno.land/x/case@2.1.1/mod.ts";

const log = {
  info: (s: string) => console.log("👉 " + colors.green(s)),
  error: (s: string) => console.error("💥 " + colors.red(s)),
  warning: (s: string) => console.error("🐓 " + colors.yellow(s)),
};

const args = parse(Deno.args);

if (args.help || args._.length < 1) {
  console.log(`Usage: ${path.basename(Deno.mainModule)} [PROJECT-NAME]`);
  Deno.exit(1);
}

try {
  customizeProject(args._[0].toString());
} catch (error) {
  log.error(error.message);
  Deno.exit(1);
}

Deno.exit(0);

function customizeProject(projectName: string) {
  const binFile = new File("src/bin/rust_cli_quickstart.rs").rename(
    snakeCase(projectName) + ".rs"
  );
  const libFile = new File("src/lib.rs");
  const benchFile = new File("benches/benchmarks.rs");
  const cargoTomlFile = new File("Cargo.toml");
  const launchFile = new File(".vscode/launch.json");

  binFile.write(
    binFile
      .text()
      .replaceAll(RegExp("rust_cli_quickstart", "gm"), snakeCase(projectName))
      .replaceAll(RegExp("RustCliQuickStart", "gm"), pascalCase(projectName))
  );

  libFile.write(
    libFile
      .text()
      .replaceAll(RegExp("RustCliQuickStart", "gm"), pascalCase(projectName))
  );

  benchFile.write(
    benchFile
      .text()
      .replaceAll(RegExp("rust_cli_quickstart", "gm"), snakeCase(projectName))
      .replaceAll(RegExp("RustCliQuickStart", "gm"), pascalCase(projectName))
  );

  cargoTomlFile.write(
    cargoTomlFile
      .text()
      .replaceAll(RegExp("rust_cli_quickstart", "gm"), snakeCase(projectName))
  );

  launchFile.write(
    launchFile
      .text()
      .replaceAll(RegExp("rust_cli_quickstart", "gm"), snakeCase(projectName))
  );

  log.info(
    "Don't forget to update the authors, description and repository in the Cargo.toml file"
  );
}
