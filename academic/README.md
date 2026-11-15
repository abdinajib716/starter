# Academic

Study material for this codebase, written as the project is built (rule: [`teacher_Agent.md`](../teacher_Agent.md)).

| File | Contents |
|---|---|
| [LEARNING.md](LEARNING.md) | One entry per finished feature: real code, numbered line notes, concepts, an exercise |
| [CONCEPTS.md](CONCEPTS.md) | Each Rust / Dioxus / library concept explained once, linked from the entries |

## How to study an entry

1. **Read** the entry's code and line notes. Open each linked concept you do not already know.
2. **Hand-write** the code in a practice branch, so `m0/foundation` (or the current branch) stays clean:
   ```sh
   git switch -c practice/entry-6        # one branch per exercise
   ```
   Open the file named above the code block, delete the quoted lines, and rewrite them from memory
   (use the line notes, not the original).
3. **Check** your version:
   ```sh
   just lint          # formatting + clippy for server and wasm
   just test          # database tests
   just dev           # run it; follow the entry's verification steps
   git diff           # compare your code with the original, line by line
   ```
4. **Try it**: make the small change in the entry and confirm you see the stated result.
5. **Reset** when done:
   ```sh
   git checkout -- . && git switch - && git branch -D practice/entry-6
   ```

Line numbers refer to the commit named at the top of each milestone section; a later commit may
have moved the code, so `git show <commit>:<path>` shows the exact version quoted.
