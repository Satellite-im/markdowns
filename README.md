# Markdowns
- provides a function called `text_to_html` which parses a subset of markdown, replaces it with html tags, and returns the string. 
handles bold, italics, strikethrough, and code. also transforms text into emojis too, ignoring text within code segments. 
- provides a function to detects if a string consists of a single emoji and wraps it in a `span`. 
- provides the text to emoji function as a standalone

## Supported markdown
 - italics
     - `*x*`
     - `_x_`
 - bold
     - `**x**`
     - `__x__`
 - strikethrough
     - `~~x~~`
 - code
     - `int a = 0;`
     - ```int a = 0;```
 - multiline code
     ```
     int a = 0;
     int b = 0;
     ```
 - multiline code with a language
     ```rust
     let a = 0;
     let b = 0;
     ```
- headings
    - `# heading title`
    - `## heading title`
    - ...
    - `##### heading title`