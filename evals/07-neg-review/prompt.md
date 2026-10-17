---
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Write, Edit, Agent, Bash]
model: opus
runs: 3
plugins: [../../plugins/celis]
---
Review this function and tell me its bugs. Do not change anything.

```python
def average(nums):
    total = 0
    for i in range(1, len(nums)):
        total += nums[i]
    return total / len(nums)
```
