%nonterminal _chunk__block__statList
%nonterminal _chunk__block__statList__stat
%nonterminal _name__nameList
%nonterminal nameList
%nonterminal exprList
%nonterminal _exprList__expr
%nonterminal _NewAxiom

%axiom _NewAxiom

%terminal NAME
%terminal IN
%terminal IF
%terminal LET
%terminal SHARP
%terminal PERCENT
%terminal STRING
%terminal DO
%terminal EQ
%terminal TRUE
%terminal DIVIDE
%terminal XEQ
%terminal GOTO
%terminal GT
%terminal AND
%terminal ELSE
%terminal ENDFILE
%terminal UMINUS
%terminal FUNCTION
%terminal DOT2
%terminal NOT
%terminal EQ2
%terminal CARET
%terminal LBRACK
%terminal SEMIFIELD
%terminal RPAREN
%terminal RBRACE
%terminal OR
%terminal PLUS
%terminal NEQ
%terminal LPARENFUNC
%terminal END
%terminal ASTERISK
%terminal THEN
%terminal COMMA
%terminal COLON2
%terminal FALSE
%terminal SEMI
%terminal NUMBER
%terminal REPEAT
%terminal MINUS
%terminal COLON
%terminal DOT
%terminal UNTIL
%terminal ELSEIF
%terminal RBRACK
%terminal LBRACE
%terminal RPARENFUNC
%terminal GTEQ
%terminal LPAREN
%terminal NIL
%terminal LTEQ
%terminal RETURN
%terminal FOR
%terminal WHILE
%terminal BREAK
%terminal LT
%terminal DOT3
%terminal _DELIM

%%

nameList : nameList COMMA _name__nameList 
	| _name__nameList COMMA _name__nameList 
	;

_NewAxiom : _chunk__block__statList 
	| _chunk__block__statList__stat 
	;

_exprList__expr : STRING 
	| FALSE 
	| NUMBER 
	| NIL 
	| TRUE 
	;

_name__nameList : NAME 
	;

_chunk__block__statList__stat : LET nameList 
	| LET _name__nameList XEQ _exprList__expr 
	| LET _name__nameList 
	| LET nameList XEQ exprList 
	| LET nameList XEQ _exprList__expr 
	| LET _name__nameList XEQ exprList 
	;

_chunk__block__statList : SEMI 
	| _chunk__block__statList SEMI _chunk__block__statList__stat 
	| _chunk__block__statList__stat SEMI 
	| _chunk__block__statList__stat SEMI _chunk__block__statList__stat 
	| _chunk__block__statList SEMI 
	;

exprList : exprList COMMA _exprList__expr 
	| _exprList__expr COMMA _exprList__expr 
	;

