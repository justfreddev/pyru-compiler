# Pyru Compiler

A simple compiler based off the [Pyru language](https://github.com/justfreddev/pyru-interpreter/blob/master/src/grammar.ebnf) built to learn compiler design. The current implementation follows this design:

Lexical analysis -> Syntax analysis -> Semantic analysis -> Build CFG -> Constant propogation -> Constant folding -> Liveliness analysis & Dead code elimination -> CFG Cleanup -> Bytecode generation -> VM execution

The playground for the language can be found [here](https://pyru-playground.vercel.app/).