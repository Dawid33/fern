%nonterminal chunk
%nonterminal statList
%nonterminal stat
%nonterminal elseIfBlock
%nonterminal exprThenElseIfB
%nonterminal exprThen
%nonterminal name
%nonterminal retStat
%nonterminal label
%nonterminal funcName
%nonterminal nameDotList
%nonterminal varList
%nonterminal var
%nonterminal nameList
%nonterminal exprList
%nonterminal expr
%nonterminal logicalOrExp
%nonterminal logicalAndExp
%nonterminal relationalExp
%nonterminal additiveExp
%nonterminal multiplicativeExp
%nonterminal unaryExp
%nonterminal caretExp
%nonterminal baseExp
%nonterminal prefixExp
%nonterminal functionCall
%nonterminal functionDef
%nonterminal parList
%nonterminal tableConstructor
%nonterminal fieldList
%nonterminal fieldListBody
%nonterminal field
%nonterminal typeExpr
%nonterminal ptrTypeStart
%nonterminal label
%nonterminal assemblyInstruction
%nonterminal pipeList 
%nonterminal textBlock

%axiom chunk

%terminal ENDFILE
%terminal RETURN
%terminal SEMI
%terminal COLON
%terminal COLON2
%terminal DOT
%terminal DOT3
%terminal COMMA
%terminal LBRACK
%terminal RBRACK
%terminal LBRACE
%terminal RBRACE
%terminal LPAREN
%terminal RPAREN
%terminal EQ
%terminal BREAK
%terminal GOTO
%terminal DO
%terminal END
%terminal WHILE
%terminal REPEAT
%terminal UNTIL
%terminal IF
%terminal THEN
%terminal ELSEIF
%terminal ELSE
%terminal FOR
%terminal IN
%terminal FUNCTION
%terminal LET
%terminal NIL
%terminal FALSE
%terminal TRUE
%terminal NUMBER
%terminal STRING
%terminal NAME
%terminal PLUS
%terminal MINUS
%terminal ASTERISK
%terminal DIVIDE
%terminal CARET
%terminal PERCENT
%terminal LT
%terminal GT
%terminal LTEQ
%terminal GTEQ
%terminal EQDOUBLE
%terminal NEQ
%terminal AND
%terminal OR
%terminal NOT
%terminal UMINUS
%terminal SHARP
%terminal SEMIFIELD
%terminal EQ
%terminal QUESTIONMARK
%terminal STRUCT
%terminal COMMENT
%terminal LABEL
%terminal MOV
%terminal LEA 
%terminal SYSCALL
%terminal PUSH
%terminal POP
%terminal SUB
%terminal PIPE
%terminal CHARS

%%

chunk : statList
	;

statList : stat
	| SEMI
	| stat SEMI
	| statList SEMI stat
	| statList SEMI
	;

stat :  LBRACK pipeList RBRACK textBlock
	| LBRACK CHARS RBRACK textBlock
	;

pipeList : CHARS PIPE CHARS
	| pipeList PIPE CHARS
	;

textBlock : LBRACK CHARS RBRACK 
	| CHARS LBRACK CHARS RBRACK 
	| stat
	; 

functionCall : baseExp LPAREN exprList RPAREN
	| baseExp LPAREN expr RPAREN
	| baseExp LPAREN RPAREN
	;

retStat : RETURN SEMI
	| RETURN exprList SEMI
	| RETURN expr SEMI
	| RETURN
	| RETURN exprList
	| RETURN expr
	;

exprList : expr COMMA expr
	| exprList COMMA expr
	;

expr : logicalOrExp
	;

logicalOrExp : logicalAndExp
	| logicalOrExp OR logicalAndExp
	;

logicalAndExp : relationalExp
	| logicalAndExp AND relationalExp
	;

relationalExp : additiveExp
	| relationalExp LT additiveExp
	| relationalExp GT additiveExp
	| relationalExp LTEQ additiveExp
	| relationalExp GTEQ additiveExp
	| relationalExp NEQ additiveExp
	| relationalExp EQDOUBLE additiveExp
	;

additiveExp : multiplicativeExp
	| additiveExp PLUS multiplicativeExp
	| additiveExp MINUS multiplicativeExp
	;

multiplicativeExp : unaryExp
	| multiplicativeExp ASTERISK unaryExp
	| multiplicativeExp DIVIDE unaryExp
	| multiplicativeExp PERCENT unaryExp
	;

unaryExp : caretExp
	| NOT unaryExp
	| SHARP unaryExp
	| UMINUS unaryExp
	;

caretExp : baseExp
	| baseExp CARET caretExp
	;

baseExp : NIL
	| FALSE
	| TRUE
	| NUMBER
	| STRING
	| NAME
	| TEXT
	| functionDef
	| prefixExp
	;

prefixExp : nameDotList
	| functionCall
	| LPAREN expr RPAREN
	;

fieldList : fieldListBody
	| fieldListBody COMMA
	;

fieldListBody : field
	| fieldListBody COMMA field
	;

field : baseExp COLON baseExp
	;

varList : var COMMA var
	| varList COMMA var
	;

funcName : nameDotList
	| nameDotList COLON baseExp
	;

nameDotList : baseExp DOT baseExp
	| nameDotList DOT baseExp
	;
	
