---
name: train
description: Teaches celis the person's taste for one kind of thing, such as a logo, an error message or a commit message, from 4 variants the person marks good, bad or neither. Use when the person runs /celis:train <kind of thing>, or asks to teach celis what they like.
---

The text after `/celis:train` names one kind of thing. You make variants, the person marks them, and
their marks become one pattern. Add it as one ask, as the card says.

1. Run `celis search -- "<the kind of thing>"` and open the examples of each pattern it gives, so the
   variants do not ask again what the person already showed.
2. Make 4 variants at low detail, each a file in a folder for the ask, such as `train/1.svg`. Make
   them differ in one or two ways that matter for this kind of thing, such as tone, density or colour,
   and say in one line each how it differs.
3. Show the person the 4 variants, and ask them to mark each `good`, `bad` or `neither`, with one line
   of why. Wait for their marks.
4. Write one pattern in the person's words, with `decide --path <the files it acts on> --good <each
   good file> --bad <each bad file> -- "When <the kind of thing> is made: <what the good ones share>,
   because <the person's why>."` When it adds to a pattern that search gave, replace that pattern with
   `--replaces <its id>`.
5. Close the ask. In the end, give the pattern's id and its text, and offer one more round for a point
   the marks left open.
