---
type: regex
target: trace
---
^(?:(?!"name":"(?:Write|Edit)")[\s\S])*?(?:"command":"(?:[^"\\]|\\.)*celis step add(?:[^"\\]|\\.)*--recalled [0-9a-f]{4,}|(?:"command":"(?:[^"\\]|\\.)*celis (?:patterns|grep [0-9a-f]{4,})|"subagent_type":"celis:recall"))
