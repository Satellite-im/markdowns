# Markdowns
- provides a function called `text_to_html` which parses a subset of markdown, replaces it with html tags, and returns the string. 
handles bold, italics, strikethrough, and code. 
- also returns a vec of ranges, each range is a substring that isn't a code segment. This allows for optional transformation of emojis.
- provides a function to turn ascii emojis into unicode emojis. 

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