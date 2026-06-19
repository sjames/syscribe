---
id: TC-TRS-PUML-021
name: "PlantUML tool resolution order is respected by plantuml render"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-051]
tags: [diagram, plantuml, render]
---

```gherkin
Feature: plantuml render — PlantUML tool resolution

  Scenario: --jar flag takes precedence over all other resolution methods
    Given PLANTUML_JAR is set to /some/other/plantuml.jar
    And [plantuml] jar = "/yet/another.jar" in .syscribe.toml
    When syscribe -m <root> plantuml render --jar /explicit/plantuml.jar is run
    Then the tool is invoked as "java -jar /explicit/plantuml.jar -tsvg ..."
    And no other resolution method is attempted

  Scenario: [plantuml] jar config key is used when --jar is absent
    Given [plantuml] jar = "/configured/plantuml.jar" in .syscribe.toml
    And no --jar flag is passed
    When syscribe -m <root> plantuml render is run
    Then the tool is invoked as "java -jar /configured/plantuml.jar -tsvg ..."

  Scenario: PLANTUML_JAR env variable is used when config jar is absent
    Given PLANTUML_JAR=/env/plantuml.jar is set in the environment
    And no --jar flag and no [plantuml] jar key
    When syscribe -m <root> plantuml render is run
    Then the tool is invoked as "java -jar /env/plantuml.jar -tsvg ..."

  Scenario: plantuml on PATH is used as final fallback
    Given no --jar flag, no [plantuml] jar, no PLANTUML_JAR env variable
    And plantuml is available on PATH
    When syscribe -m <root> plantuml render is run
    Then the tool is invoked as "plantuml -tsvg ..."

  Scenario: Clear error when PlantUML cannot be found
    Given no --jar flag, no [plantuml] jar, no PLANTUML_JAR, and plantuml not on PATH
    When syscribe -m <root> plantuml render is run
    Then the process exits non-zero
    And stderr explains the resolution order and how to configure a JAR path
```
