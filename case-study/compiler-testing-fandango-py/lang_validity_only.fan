from lang_constraints import num_violations_validity_only

<start> ::= <prog_statements> ;

<prog_statements> ::= <prog_stmt> | <prog_stmt> "\n" <prog_statements> ;

<prog_stmt> ::= <decl> ";" | <fn_def> ;

<statements> ::= <stmt> "\n" | <stmt> "\n" <statements> ;

<stmt> ::= <decl> ";" | <assignment> ";" | <struct_def> ";" | <expr_stmt> ";" | <return_stmt> ";" ;

<expr_stmt> ::= <expr> ;

<ttype> ::= <basic_type> | <struct_type> ;
<basic_type> ::= "int" | "float" | "double" | "bool" | "char" | "void" ;

<struct_type> ::= "struct" <sep> <struct_name> ;
<struct_name> ::= <letter> <alnums> ;

<decl> ::= <ttype> <sep> <var_name> <sep> <decl_rhs_e> ;
<decl_rhs_e> ::= <decl_rhs> | <e> ;
<decl_rhs> ::= "=" <sep> <expr> ;

<struct_def> ::= "struct" <sep> <struct_name> <sep> "{" "\n" <field_def_list_e> "\n" "}" ;
<field_def_list_e> ::= <field_def_list> | <e> ;
<field_def_list> ::= <ttype> <sep> <field_name> ";" | <ttype> <sep> <field_name> ";" "\n" <field_def_list> ;
<field_name> ::= <letter> <alnums> ;

<assignment> ::= <var_access> <sep> <assign_op> <sep> <expr> ;
<assign_op> ::= "=" | "+=" | "-=" ;

<fn_def> ::= <ttype> <sep> <fn_kwd> <sep> <fn_name> "(" <param_list_e> ")" <sep> "{" <sep> <fn_body_e> <sep> "}" ;
<fn_kwd> ::= "" ;
<fn_name> ::= <letter> <alnums> ;
<param_list_e> ::= <param_list> | <e> ;
<param_list> ::= <param> | <param> "," <sep> <param_list> ;
<param> ::= <ttype> <sep> <param_name> ;
<param_name> ::= <var_name> ;
<fn_body_e> ::= <statements> | <e> ;

<return_stmt> ::= <return_kwd> | <return_kwd> <sep> <expr> ;
<return_kwd> ::= "return" ;

<expr> ::= <arith_expr> | <expr_unit> ;
<expr_unit> ::= <var_access> | <value> | <fn_call> | <struct_expr> | <struct_access> ;
<fn_call> ::= <fn_name> "(" <arg_list_e> ")" ;
<arg_list_e> ::= <arg_list> | <e> ; 
<arg_list> ::= <arg> | <arg> "," <sep> <arg_list> ;
<arg> ::= <expr> ;

<struct_expr> ::= "{" "\n" <expr_list_e> "\n" "}" ;
<expr_list_e> ::= <expr_list> | <e> ;
<expr_list> ::= <expr> | <expr> "," "\n" <expr_list> ;

<arith_expr> ::= <binop> | <unop> ;

<binop> ::= <expr_unit> <sep> <binop_op> <sep> <expr> ;
<unop> ::= <unop_op> <expr> ;

<binop_op> ::= "+" | "-" | "/" | "*" | "%" | "^" | "==" | "!=" | "<=" | ">=" | "<" | ">" | "&&" | "||" ;
<unop_op> ::= "-" | "!" ;

<var_access> ::= <var_name> ;
<var_name> ::= <letter> | <letter> <alnums> ;

<struct_access> ::= <var_name> "." <field_name> ;

<alnums> ::= <alnum> | <alnum> <alnums> ;
<alnum> ::= <letter> | <digit> ;
<letter> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" ;
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

<value> ::= <bool_val> | <num_val> | <string_val> ;

<bool_val> ::= "true" | "false" ;
<num_val> ::= <int> | <float> | <double> ;
<int> ::= <digit> | <digit> <int> ;
<float> ::= <int> "." <int> "f" ;
<double> ::= <int> "." <int> ;
<string_val> ::= "\' \'" | "\'" <alnum> "\'" ;
<sep> ::= " " ;
<e> ::= " " ;

where num_violations_validity_only(<start>) == 0 ;