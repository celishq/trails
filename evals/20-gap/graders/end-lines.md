---
type: llm
focus: trace
---
Read the agent's last message to the person. Pass only if it opens with a "Done:" line, holds no line whose value is empty or "none" (such as "Turn: none"), lists under "Not proven" each step that the total line counts as a claim, and ends with the total line that `celis step done` printed, which starts with "celis · ".
