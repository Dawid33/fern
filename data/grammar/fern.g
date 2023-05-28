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
%nonterminal concatExp
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
%terminal DOT2
%terminal LT
%terminal GT
%terminal LTEQ
%terminal GTEQ
%terminal EQ2
%terminal NEQ
%terminal AND
%terminal OR
%terminal NOT
%terminal UMINUS
%terminal SHARP
%terminal LPARENFUNC
%terminal RPARENFUNC
%terminal SEMIFIELD
%terminal EQ
%terminal QUESTIONMARK
%terminal STRUCT

%%

chunk : statList
	;

statList : stat
	| SEMI
	| stat SEMI
	| statList SEMI stat
	| statList SEMI
	;

stat :  varList EQ exprList
	| functionCall
	| retStat
	| LBRACE statList RBRACE
	| LBRACE RBRACE
	| WHILE expr LBRACE statList RBRACE
	| WHILE expr LBRACE RBRACE
	| FUNCTION funcName LBRACK nameList RBRACK LBRACE statList RBRACE
	| IF exprThen RBRACE
	| IF exprThen RBRACE elseIfBlock
	| STRUCT name LBRACE RBRACE
	| STRUCT name LBRACE fieldList RBRACE
	| FUNCTION funcName LPARENFUNC RPARENFUNC LBRACE statList RBRACE
	| FUNCTION funcName LPARENFUNC nameList RPARENFUNC LBRACE statList RBRACE
	| FUNCTION funcName LPARENFUNC nameList RPARENFUNC LBRACE RBRACE
	| FUNCTION funcName LPARENFUNC RPARENFUNC LBRACE RBRACE
	| FOR nameList IN exprList LBRACE statList RBRACE
	| FOR nameList IN exprList LBRACE RBRACE
	| LET nameList COLON typeExpr EQ exprList
	| LET nameList COLON typeExpr
	| LET nameList EQ exprList
	| LET nameList
	;

functionCall : prefixExp LPAREN exprList RPAREN
	| prefixExp LPAREN RPAREN
	;

typeExpr : name
    | ASTERISK name
    | QUESTIONMARK
    ;

retStat : RETURN SEMI
	| RETURN exprList SEMI
	| RETURN
	| RETURN exprList
	;

elseIfBlock : ELSEIF expr LBRACE statList RBRACE
	| ELSEIF expr LBRACE RBRACE
	| ELSEIF expr LBRACE RBRACE elseIfBlock
	| ELSEIF expr LBRACE statList RBRACE elseIfBlock
	| ELSE LBRACE RBRACE
	| ELSE LBRACE statList RBRACE
	;

exprThen : expr LBRACE statList
	| expr LBRACE
	;

name : NAME
	;

nameList : NAME
	| nameList COMMA name
	;

exprList	: expr
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

relationalExp : concatExp
	| relationalExp LT concatExp
	| relationalExp GT concatExp
	| relationalExp LTEQ concatExp
	| relationalExp GTEQ concatExp
	| relationalExp NEQ concatExp
	| relationalExp EQ2 concatExp
	;

concatExp : additiveExp
	| additiveExp DOT2 concatExp
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
	| functionDef
	| prefixExp
	;

prefixExp : var
	| functionCall
	| LPAREN expr RPAREN
	;

fieldList : fieldListBody
	| fieldListBody COMMA
	;

fieldListBody : field
	| fieldListBody COMMA field
	;

field : name COLON typeExpr
	;

var : NAME
	| prefixExp DOT NAME
	;

varList : var
	| varList COMMA var
	;

funcName : nameDotList
	| nameDotList COLON name
	;

nameDotList : NAME
	| nameDotList DOT NAME
	;

nameList : NAME
	| nameList COMMA name
	;
