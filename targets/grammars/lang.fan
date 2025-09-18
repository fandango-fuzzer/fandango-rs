<start> ::= <statements> ;

<statements> ::= <stmt> | <stmt> "\n" <statements> ;

<stmt> ::= <decl> | <assignment> | <fn_def> | <expr> | <return> ;

<decl> ::= <decl_kwd> <sep> <var_name> <sep> <decl_rhs_opt> ;
<decl_rhs_opt> ::= <decl_rhs> | "" ;
<decl_kwd> ::= "let" | "const" ;
<decl_rhs> ::= "=" <sep> <expr> ;

<assignment> ::= <var_access> <sep> <assign_op> <sep> <expr> ;
<assign_op> ::= "=" | "+=" | "-=" ;

<fn_def> ::= <fn_kwd> <sep> <fn_name> "(" <param_list_e> ")" <sep> "{" <sep> <fn_body> <sep> "}" ;
<fn_kwd> ::= "function" ;
<fn_name> ::= <letter> <alnums> ;
<param_list_e> ::= <empty> | <param_list> ;
<param_list> ::= <param_name> | <param_name> "," <sep> <param_list> ;
<param_name> ::= <letter> <alnums> ;
<fn_body> ::= <empty> | <statements> ;

<return> ::= <return_kwd> | <return_kwd> <sep> <expr> ;
<return_kwd> ::= "return" ;

<expr> ::= <arith_expr> | <expr_unit> ;
<expr_unit> ::= <var_access> | <value> | <fn_call> ;
<fn_call> ::= <var_access> "(" <arg_list> ")" | <var_access> "(" <empty> ")" ;
<arg_list> ::= <arg> | <arg> "," <sep> <arg_list> ;
<arg> ::= <expr> ;

<arith_expr> ::= <binop> | <unop> ;

<binop> ::= <expr_unit> <sep> <binop_op> <sep> <expr> ;
<unop> ::= <unop_op> <expr> ;

<binop_op> ::= "+" | "-" | "/" | "*" ;
<unop_op> ::= "-" ;

<var_access> ::= <var_name> ;
<var_name> ::= <letter> | <letter> <alnums> ;

<alnums> ::= <alnum> | <alnum> <alnums> ;
<alnum> ::= <letter> | <digit> ;
<letter> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" ;
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

<value> ::= <bool_val> | <num_val> | <string_val> ;

<bool_val> ::= "true" | "false" ;
<num_val> ::= <int> ;
<int> ::= <digit> | <digit> <int> ;
<string_val> ::= "\"\"" | "\"" <alnums> "\"" ;
<sep> ::= " " ;
<empty> ::= "" ;