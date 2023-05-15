%nonterminal _chunk__expr
%nonterminal _chunk__A
%nonterminal _NewAxiom

%axiom _NewAxiom

%terminal IN
%terminal EQ
%terminal FUNCTION
%terminal BREAK
%terminal FOR
%terminal LPARENFUNC
%terminal LET
%terminal UMINUS
%terminal REPEAT
%terminal NUMBER
%terminal COLON
%terminal RPAREN
%terminal PLUS
%terminal SEMIFIELD
%terminal THEN
%terminal GTEQ
%terminal NOT
%terminal ELSE
%terminal RBRACK
%terminal UNTIL
%terminal NEQ
%terminal FALSE
%terminal SHARP
%terminal PERCENT
%terminal GOTO
%terminal LBRACK
%terminal OR
%terminal DOT3
%terminal TRUE
%terminal COLON2
%terminal ENDFILE
%terminal NIL
%terminal LTEQ
%terminal COMMA
%terminal DOT2
%terminal DO
%terminal CARET
%terminal LBRACE
%terminal DIVIDE
%terminal LT
%terminal SEMI
%terminal NAME
%terminal STRING
%terminal RBRACE
%terminal RPARENFUNC
%terminal DOT
%terminal RETURN
%terminal END
%terminal AND
%terminal ASTERISK
%terminal MINUS
%terminal GT
%terminal EQ2
%terminal LPAREN
%terminal XEQ
%terminal ELSEIF
%terminal WHILE
%terminal IF
%terminal _DELIM

%%

_chunk__expr : DOT NOT 
	;

_chunk__A : COMMA 
	;

_NewAxiom : _chunk__A 
	| _chunk__expr 
	;

