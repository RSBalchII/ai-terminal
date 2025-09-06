# Task List: Coda A-001 (The Architect)
- **Version:** 1.0
- **Status:** To Do
- **Plan Link:** `specs/Coda-A-001/plan.md`
- **Date:** 2025-09-06

---

## Phase 1: Foundational Persona Implementation
*Goal: Establish the basic operational capability of the A-001 agent.*

- [ ] **P1.1: Constitution Finalization**
    - [ ] Create the final `Coda-A-001.poml` file in a dedicated agent directory.
    - [ ] Verify the XML is well-formed.
- [ ] **P1.2: Core Prompt Engineering**
    - [ ] Draft the initial system prompt (`sys_prompt.md`) that incorporates the core metaphor, values, and directives from the finalized POML.
- [ ] **P1.3: Basic I/O Loop**
    - [ ] Create a simple Python or Rust script (`main.py` or `main.rs`) to load the system prompt and POML.
    - [ ] Implement a basic `while True` loop for interactive command-line interaction.
    - [ ] Test the basic I/O to ensure the agent can receive a prompt and generate a text response.

---

## Phase 2: Core Protocol Development
*Goal: Implement the agent's primary functional capabilities.*

- [ ] **P2.1: Develop `Constitutional_Analysis` Protocol**
    - [ ] Engineer a multi-step prompt chain that instructs the agent to analyze an initial user request and formulate Socratic questions to resolve ambiguity.
    - [ ] Create a test file with sample "vague requests" to validate the questioning logic.
- [ ] **P2.2: Develop POML Generation Engine**
    - [ ] Develop the main generative prompt that takes a structured set of elicited requirements as input.
    - [ ] Add explicit instructions to the prompt to output only valid XML within a markdown code block for easy parsing.
    - [ ] Test the generator with a set of well-defined requirements and verify the output is syntactically correct.
- [ ] **P2.3: Develop `Integrity_Validation` Protocol**
    - [ ] Draft the "Critic" system prompt, instructing the agent to act as a logical reviewer checking for internal contradictions in a given POML.
    - [ ] Implement the two-step logic: the output from the Generation engine (P2.2) is fed as input to the Critic prompt.
    - [ ] Test by providing a POML draft with a logical flaw (e.g., conflicting protocols) and verifying the Critic identifies the issue.

---

## Phase 3: Knowledge Base & Refinement Loop
*Goal: Enable the agent to learn and improve over time.*

- [ ] **P3.1: Implement Knowledge Base**
    - [ ] Create the initial `knowledge_base.md` file.
    - [ ] Populate it with 3-5 examples of high-quality POML snippets (e.g., a well-defined protocol, a powerful core metaphor, a clear set of values).
    - [ ] Update the core prompt to instruct the agent to reference this file for inspiration and best practices.
- [ ] **P3.2: Implement `Iterative_Refinement` Protocol**
    - [ ] Design a prompt that takes an existing draft POML and a natural language change request as input.
    - [ ] Test the refinement prompt with several modification requests (e.g., "Change the agent's name to 'Observer'", "Add a protocol for data logging").

---

## Phase 4: Testing & Integration
*Goal: Verify the agent's effectiveness and integrate it into our standard workflow.*

- [ ] **P4.1: Develop Evaluation Suite**
    - [ ] Create a `test_suite.md` file.
    - [ ] Add at least three distinct persona requests (e.g., a "code reviewer" agent, a "creative writer" agent, a "data analyst" agent), each with defined success criteria.
    - [ ] Run the full A-001 agent against the test suite and document the results, comparing outputs to the success criteria.
- [ ] **P4.2: Workflow Integration**
    - [ ] Define a command (e.g., `/architect create <prompt>`) for invocation from the main AI-Terminal.
    - [ ] Implement the function or script that allows the AI-Terminal to call the A-001 agent with the user's prompt.
    - [ ] Perform a final end-to-end test, from issuing the command in the terminal to receiving the generated POML file.