# Container-Use Agent Rules

## Core Rules (MANDATORY - NO EXCEPTIONS)

ALWAYS use ONLY Environments for ANY and ALL file, code, or shell operations—NO EXCEPTIONS—even for simple or generic requests.

DO NOT install or use the git cli with the environment_run_cmd tool. All environment tools will handle git operations for you. Changing ".git" yourself will compromise the integrity of your environment.

You MUST inform the user how to view your work using `container-use log <env_id>` AND `container-use checkout <env_id>`. Failure to do this will make your work inaccessible to others.

---

## Extended Stability Rules

### Environment Lifecycle Management

1. **Environment Creation**
   - ALWAYS create a new environment at the start of a new task/issue
   - Record the `env_id` immediately after creation
   - Use descriptive environment names matching the task (e.g., `feature-issue-8-sound`)

2. **Environment Persistence**
   - NEVER abandon an environment due to errors
   - If an operation fails, diagnose and retry within the SAME environment
   - Use `environment_open` to reconnect to existing environments

3. **Environment Reuse Rules**
   | Situation | Action |
   |-----------|--------|
   | Same issue, continuing work | Reuse existing environment via `environment_open` |
   | PR review feedback/fixes | Reuse the SAME environment (do NOT create new) |
   | New issue/feature | Create NEW environment |
   | Fix branch for different issue | Create NEW environment |

### Crash Recovery Protocol

When encountering errors or crashes:

1. **DO NOT** fall back to direct host file operations
2. **DO NOT** abandon the container-use workflow
3. **INSTEAD**, follow this recovery flow:
   ```
   a. Check environment status with `environment_list`
   b. Reopen the environment with `environment_open(env_id)`
   c. Verify file state with `environment_file_list`
   d. Continue work within the environment
   ```

4. **If environment is corrupted:**
   ```
   a. Create a NEW environment with the same branch
   b. The git state will be preserved from the remote
   c. Continue work in the new environment
   ```

### Forbidden Actions (HARD BLOCKS)

| Action | Why It's Forbidden |
|--------|-------------------|
| Direct file read/write on host | Bypasses container isolation |
| Using `bash` for file operations | Must use environment_* tools |
| Abandoning environment on error | Loses work and context |
| Creating environment without recording env_id | Cannot recover later |
| Using git CLI in environment_run_cmd | Corrupts environment git state |

### Required Outputs

After ANY container-use session, ALWAYS provide:

```
## Environment Info
- Environment ID: `<env_id>`
- View logs: `container-use log <env_id>`
- Checkout code: `container-use checkout <env_id>`
```

### Error Handling Escalation

1. **First failure**: Retry the operation
2. **Second failure**: Check environment state, diagnose issue
3. **Third failure**: Create checkpoint, attempt alternative approach
4. **Persistent failure**: Report to user WITH environment ID for manual recovery

NEVER silently switch to non-container-use operations.

---

## Quick Reference

### Common Operations

| Task | Tool to Use |
|------|-------------|
| Create new environment | `environment_create` |
| Reopen existing environment | `environment_open` |
| List files | `environment_file_list` |
| Read file | `environment_file_read` |
| Write file | `environment_file_write` |
| Edit file | `environment_file_edit` |
| Run command | `environment_run_cmd` |
| Save progress | `environment_checkpoint` |

### Environment Naming Convention

```
<type>-<issue>-<feature>
```

Examples:
- `feature-issue-8-sound-playback`
- `fix-issue-6-ci-failure`
- `refactor-notification-module`
