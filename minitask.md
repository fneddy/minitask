
## Minitask Commands

| Command | Description | Default | Optional |
|---------|-------------| ------- | -------- |
| `minitask --json-out` | all output will be as json | No | Yes |
| `minitask --json-in` | all input will be by stdin as json | No | Yes |
| `minitask --file [task.toml]` | define the tasks file to use | tasks.toml | Yes |
| `minitask list` | List all tasks | - |- |
| `minitask list --state [state]` | List tasks in specific state | - | Yes |
| `minitask list --epic [epic]` | List tasks in specific epic | - | Yes |
| `minitask list --verbose` | List tasks with full details | - | Yes |
| `minitask show [task-id]` | Show complete task details | - | No |
| `minitask new "content"` | Create new task | - | Either |
| `minitask new -` | Create new task with content from stdin | - | Either |
| `minitask edit state [task-id] [state]` | Change task state | - | No |
| `minitask edit content [task-id] "content"` | Replace task content | - | No |
| `minitask add content [task-id] "content"` | Append to task content | - | No |
| `minitask add depends-on [task-id] [other-id]` | Add task dependency | - | No |
| `minitask del depends-on [task-id] [other-id]` | Remove task dependency | - | No |
| `minitask add epic [task-id] [epic-id]` | Add task to epic | - | No |
| `minitask del epic [task-id] [epic-id]` | Remove task from epic | - | No |
| `minitask claim [new-state]` | Claim next task from todo (default) and move to new state | - | - |
| `minitask claim [new-state] --state [state]` | Claim next task from specified state | - | Yes |
| `minitask claim [new-state] --epic [epic]` | Claim next task from specific epic | - | Yes |
| `minitask claim [new-state] -s [state] -e [epic]` | Claim with custom state and epic filters | - | Yes |


## Minitask File format

task file format is toml
```toml
[[tasks]]
name = "TASK-1"
state = "done"
depends_on = []
epic = ["planning"]
content = """
content
"""

[[tasks]]
name = "TASK-2"
state = "done"
depends_on = ["TASK-1"]
epic = ["planning"]
content = """
content
"""
[[tasks]]
name = "TASK-3"
state = "done"
depends_on = ["TASK-2"]
epic = ["implementation"]
content = """
content
"""
```

## implementation requirements
- keep it as simple as possible
- reuse structs for clap command parsing and json input parsing
- reuse structs for toml serialization and json output
- all unittests are in src/tests.rs
- complete implementation is is src/main.rs
- integration tests are in integration_tests.sh
- integration tests call cargo run -- 
