# build-config — generate build-system artifacts from a Configuration

## SYNOPSIS
    syscribe -m <root> build-config --config <id> --format <fmt> [--prefix <p>] [--no-validate]
    syscribe -m <root> build-config --all-configs [--format json] [--prefix <p>] [--no-validate]

## DESCRIPTION
Resolves a Configuration's feature selections and parameter bindings into a set of
named build variables and emits them in the requested format. By default the
configuration is SAT-validated before output; use --no-validate to skip.

Variable resolution order (last writer wins):
  1. buildExports: on selected FeatureDefs (whenSelected / whenDeselected)
  2. parameterBindings: → buildVar: on FeatureDef parameters
  3. buildOverrides: on the Configuration

Output variables are emitted in alphabetical order for reproducible diffs.

## OPTIONS
    --config <id>      Named Configuration to project (id or qualified name).
    --format <fmt>     Output format: cmake | c-header | makefile | env | json | kconfig
                       Default: json
    --prefix <str>     Prepend <str> to every emitted variable name.
    --no-validate      Skip SAT validation before generating output.
    --all-configs      Generate output for every Configuration (JSON array, for CI matrix).

## FORMATS
    cmake      set(VAR value) — include() in CMakeLists.txt
    c-header   #define VAR value — include in C/C++ source
    makefile   VAR := value — include in Makefile
    env        export VAR=value — source in shell
    json       {"config":…, "vars":{…}} — any build system
    kconfig    CONFIG_VAR=y/n/value — Zephyr/Linux Kconfig

## EXAMPLES
    syscribe -m model/ build-config --config CONF-PREMIUM --format cmake
    syscribe -m model/ build-config --config CONF-PREMIUM --format c-header --prefix MY_
    syscribe -m model/ build-config --all-configs --format json

## SEE ALSO
    configure, feature-check, validate --config, export

## DIAGNOSTICS
    E050   Two selected features export the same variable name with conflicting values.
           Resolve by adding a buildOverrides: entry on the Configuration.
    W050   A selected feature contributes no build variable (opt-in, --deny W050).
