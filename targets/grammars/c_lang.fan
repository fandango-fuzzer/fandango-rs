<start> ::= <statements> ;

<statements> ::= <stmt> | <stmt> "\n" <statements> ;

<<<<<<< HEAD
<stmt> ::= <decl> ";" | <assignment> ";" | <fn_def> | <struct_def> ";" | <expr_stmt> ";" | <return_stmt> ";" ;

<expr_stmt> ::= <expr> ;

<type> ::= <basic_type> | <struct_type> ;
<basic_type> ::= "int" | "float" | "double" | "bool" | "char" | "void" ;
=======
<stmt> ::= <decl> ";" | <assignment> ";" | <fn_def> | <struct_def> | <expr> ";" | <return_stmt> ";" ;

<type> ::= <basic_type> | <struct_type> ;
<basic_type> ::= <int_type> | <float_type> | <double_type> | <bool_type> | <char_type> ;
<int_type> ::= <int_short> | <int_long> | <int_long_long> | <signed> <sep> "int" | "int" ;
<int_short> ::= <signed> <sep> "short" | <signed> <sep> "short" <sep> "int" | "short" <sep> "int" | "short" ;
<int_long> ::= <signed> <sep> "long" | <signed> <sep> "long" <sep> "int" | "long" <sep> "int" | "long" ;
<int_long_long> ::= <signed> <sep> "long long" | <signed> <sep> "long long" <sep> "int" | "long long" <sep> "int" | "long long" ;
<float_type> ::= "float" ;
<double_type> ::= "long" <sep> "double" | "double" ;
<bool_type> ::= "true" | "false" ;
<char_type> ::= <signed> <sep> "char" | "char" ;
<signed> ::= "signed" | "unsigned" ;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)

<struct_type> ::= "struct" <sep> <struct_name> ;
<struct_name> ::= <letter> <alnums> ;

<decl> ::= <type> <sep> <var_name> <sep> <decl_rhs_e> ;
<decl_rhs_e> ::= <decl_rhs> | <e> ;
<decl_rhs> ::= "=" <sep> <expr> ;

<struct_def> ::= "struct" <sep> <struct_name> <sep> "{" "\n" <field_def_list_e> "\n" "}" ;
<field_def_list_e> ::= <field_def_list> | <e> ;
<<<<<<< HEAD
<field_def_list> ::= <type> <sep> <field_name> ";" | <type> <sep> <field_name> ";" "\n" <field_def_list> ;
=======
<field_def_list> ::= <type> <sep> <field_name> | <type> <sep> <field_name> "," "\n" <field_def_list> ;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
<field_name> ::= <letter> <alnums> ;

<assignment> ::= <var_access> <sep> <assign_op> <sep> <expr> ;
<assign_op> ::= "=" | "+=" | "-=" ;

<fn_def> ::= <type> <sep> <fn_kwd> <sep> <fn_name> "(" <param_list_e> ")" <sep> "{" <sep> <fn_body_e> <sep> "}" ;
<<<<<<< HEAD
<fn_kwd> ::= "" ;
=======
<fn_kwd> ::= "function" ;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
<fn_name> ::= <letter> <alnums> ;
<param_list_e> ::= <param_list> | <e> ;
<param_list> ::= <param> | <param> "," <sep> <param_list> ;
<param> ::= <type> <sep> <param_name> ;
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

<<<<<<< HEAD
<struct_expr> ::= "{" "\n" <expr_list_e> "\n" "}" ;
<expr_list_e> ::= <expr_list> | <e> ;
=======
<struct_expr> ::= "{" "\n" <expr_list> "\n" "}" ;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
<expr_list> ::= <expr> | <expr> "," "\n" <expr_list> ;

<arith_expr> ::= <binop> | <unop> ;

<binop> ::= <expr_unit> <sep> <binop_op> <sep> <expr> ;
<unop> ::= <unop_op> <expr> ;

<binop_op> ::= "+" | "-" | "/" | "*" | "%" | "^" | "==" | "~=" | "<=" | ">=" | "<" | ">" | <sep> "and" <sep> | <sep> "or" <sep> | <sep> "in" <sep> ;
<unop_op> ::= "-" | <sep> "not" <sep> ;

<var_access> ::= <var_name> ;
<var_name> ::= <letter> | <letter> <alnums> ;

<struct_access> ::= <var_name> "." <field_name> ;

<alnums> ::= <alnum> | <alnum> <alnums> ;
<alnum> ::= <letter> | <digit> ;
<letter> ::= "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" ;
<digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

<value> ::= <bool_val> | <num_val> | <string_val> ;

<bool_val> ::= "true" | "false" ;
<<<<<<< HEAD
<num_val> ::= <int> | <float> | <double> ;
<int> ::= <digit> | <digit> <int> ;
<float> ::= <int> "." <int> "f" ;
<double> ::= <int> "." <int> ;
=======
<num_val> ::= <num_val_inner> | <num_val_inner> <exponent> ;
<exponent> ::= "e" <int> | "e-" <int> | "E" <int> | "E-" <int> ;
<num_val_inner> ::= <int> | <float> | <hex> ;
<hex> ::= "0x" <hex_digit>+ ;
<hex_digit> ::= "a" | "b" | "c" | "d" | "e" | "f" | <digit> ;
<int> ::= <digit> | <digit> <int> ;
<float> ::= <int> "." <int> ;
>>>>>>> 9152e20 (split lang into lua and c; more visitors; wip type checker)
<string_val> ::= "\"\"" | "\"" <alnums> "\"" ;
<sep> ::= " " ;
<e> ::= " " ;