<start> ::= <expr>;
<expr> ::= <number> "+" <expr> | <number>;
<number> ::= "0" | <non_zero><digit>*;
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