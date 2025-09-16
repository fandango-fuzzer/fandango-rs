<start> ::= <statements> ;

<statements> ::= <stmt> | <stmt> "\n" <statements> ;

<stmt> ::= <decl> | <assignment> ;

<decl> ::= <decl_kwd> <sep> <var_name> <decl_rhs_opt> ;
<decl_rhs_opt> ::= <decl_rhs> | "" ;
<decl_kwd> ::= "let" | "const" ;
<decl_rhs> ::= "=" <expr> ;

<assignment> ::= <var_name> <assign_op> <expr> ;
<assign_op> ::= "=" | "+=" | "-=" ;

<expr> ::= <arith_expr> | <expr_unit> ;
<expr_unit> ::= <var_access> | <value> ;

<arith_expr> ::= <binop> | <unop> ;

<binop> ::= <expr_unit> <binop_op> <expr> ;
<unop> ::= <unop_op> <expr> ;

<binop_op> ::= "+" | "-" | "/" | "*" ;
<unop_op> ::= "-" ;

<var_access> ::= <var_name> ;
<var_name> ::= <letter> <alnums> ;

<alnums> ::= <alnum> | <alnum> <alnums> ;
<alnum> ::= <letter> | <digit> ;
<letter> ::= "a" | "b" | "c" | "d" ;
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

<value> ::= <bool_val> | <num_val> | <string_val> ;

<bool_val> ::= "true" | "false" ;
<num_val> ::= <int> ;
<int> ::= <digit> | <digit> <int> ;
<string_val> ::= "\"\"" | "\"" <alnums> "\"" ;
<sep> ::= " " ;