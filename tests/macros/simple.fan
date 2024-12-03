<start> ::= <expr>;
<expr> ::= <number> | <number> "+" <expr>;
<number> ::= <non_zero><digit>* | "0";
<non_zero> ::=
              "1"
            | "2"
            | "3"
            | "4"
            | "5"
            | "6"
            | "7"
            | "8"
            | "9"
            ;
<digit> ::= "0" | <non_zero>;