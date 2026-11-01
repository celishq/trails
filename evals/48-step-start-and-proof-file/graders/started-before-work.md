---
type: regex
target: trace
arm: with-only
---
^(?:(?!"name":"(?:Write|Edit)"|cat > [^"\\ ]*\.py)[\s\S])*?"command":"(?:[^"\\]|\\.)*celis step start [0-9a-f]{4,}
