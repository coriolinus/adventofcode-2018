# input modifications

LALRPOP can technically not ignore whitespace if you write your own lexer,
but it was much, much easier to just add a semicolon to the end of the first
line of the input document.
