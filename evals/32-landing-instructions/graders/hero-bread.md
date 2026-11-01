---
type: regex
target: {source: file, path: index.html}
flags: i
weight: 0.1
---
<body\b[^>]*>(?:(?!</h1>)[\s\S])*?(?:sourdough|bread|loaf|loaves|dough|crust|oven|flour|bak(?:e|ed|er|ery|ing)|starter|rye|wheat|<img\b|<picture\b)|</h1>[\s\S]{0,2000}?(?:sourdough|bread|loaf|loaves|dough|crust|oven|flour|bak(?:e|ed|er|ery|ing)|starter|rye|wheat|<img\b|<picture\b)
