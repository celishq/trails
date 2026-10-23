---
type: llm
focus: trace
---
Pass only if all hold:
1. The agent made 2 variants of the logo as steps under the outer layer, and closed each with `--file`.
2. One variant explores away from the other and names what it does differently.
3. The agent chose one variant itself and went on to finish the logo, without waiting for the person.
4. Its end names the variant it chose on a "Took" line, with the blob of the other, so the person can pick after.
