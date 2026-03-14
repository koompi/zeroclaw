# Sentinel Heartbeat Protocol

Defines how Sentinel monitors, reports, and maintains awareness of system state and agent health.

## Heartbeat Cadence

| Signal | Interval | Purpose |
|---|---|---|
| **Agent health check** | Every active turn | Verify subagent is responsive before delegation |
| **Registry sweep** | Every 60s | Prune completed runs, process pending announces |
| **Memory sync** | After each synthesis | Persist key findings to long-term memory |
| **User status update** | At phase boundaries | Report progress to user at natural milestones |

## Health Indicators

### Agent Health
- **Alive**: Subagent responded within timeout
- **Stalled**: No response for > 50% of timeout — consider killing and reassigning
- **Dead**: Timeout exceeded — kill, log failure, reassign to another agent or escalate

### System Health
- **Provider availability**: Check if the LLM provider is responding
- **Memory backend**: Verify read/write operations succeed
- **Channel connectivity**: Confirm the user's channel (Telegram, etc.) is reachable
- **Token budget**: Track context window usage, trigger compaction before overflow

## Status Reporting

When Sentinel reports to the user, use this structure:

```
## Status Update

**Phase**: [current phase name]
**Progress**: [X/Y tasks complete]

| Agent | Task | Status |
|-------|------|--------|
| Forge | Implement auth | ✅ Complete |
| Oracle | Market research | 🔄 In progress |
| Prism | Design dashboard | ⏳ Queued |

**Next**: [what happens next]
**Blockers**: [any issues needing user input]
```

## Failure Recovery

1. **Subagent timeout**: Kill → reassign to same persona with simplified task
2. **Provider error**: Wait 5s → retry with same provider → fallback to alternate model
3. **Context overflow**: Trigger compaction → re-summarize → continue
4. **User disconnect**: Persist state → resume on reconnect
5. **Cascade failure** (3+ agents fail): Stop all, report to user, ask for guidance

## Sentinel Self-Check

Before each major operation, Sentinel verifies:
- [ ] Task decomposition is complete and unambiguous
- [ ] Each subtask has a clear owner (agent)
- [ ] Dependencies are ordered correctly
- [ ] No circular dependencies exist
- [ ] Token budget is sufficient for the operation
- [ ] User's channel is connected and responsive
