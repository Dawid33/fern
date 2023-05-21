%nonterminal chunk
%nonterminal chunk
%nonterminal statList
%nonterminal stat
%nonterminal elseIfBlock
%nonterminal exprThenElseIfB
%nonterminal exprThen
%nonterminal name
%nonterminal eCe
%nonterminal eCeCe
%nonterminal dot3
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

%%

chunk : statList
	;

statList : stat
	| SEMI.0
	| stat.0.0 SEMI.0
	| statList.0.1 SEMI.0 stat.0.0
	| statList.0.0 SEMI.0
	;


stat :  varList.0.0 EQ.0 exprList0.1
	| functionCall
	| retStat
	| LBRACE.0 chunk.1 RBRACE.2
	| LBRACE.0 RBRACE.1
	| WHILE.0 expr.0.0 LBRACE.0.1 chunk.0.2 RBRACE.0.3
	| WHILE.0 expr.0.0 LBRACE.0.1 RBRACE.0.2
	| FUNCTION.0 funcName.0.0 LBRACK.0.1 nameList.0.2 RBRACK.0.3 LBRACE.0.4 chunk.0.5 RBRACE.0.6
	| IF.0 exprThen.0.0 RBRACE.0.1
	| IF.0 exprThen.0.0 RBRACE.0.1 elseIfBlock.0.2
	| FUNCTION.0 funcName.0.0 LBRACK.0.1 RBRACK.0.2 LBRACE.0.3 chunk.0.4 RBRACE.0.5
	| FUNCTION.0 funcName.0.0 LBRACK.0.1 nameList.0.2 RBRACK.0.3 LBRACE.0.5 RBRACE.0.5
	| FUNCTION.0 funcName.0.0 LBRACK.0.1 RBRACK.0.2 LBRACE.0.3 RBRACE.0.4
	| FOR.0 nameList.0.0 IN.0.1 exprList.0.2 LBRACE.0.3 chunk.0.4 RBRACE.0.5
	| FOR.0 nameList.0.0 IN.0.1 exprList.0.2 LBRACE.0.3 RBRACE.0.4
	| LET.0 nameList.0.0
	| LET.0 nameList.0.0.0 EQ.0.0 exprList.0.0.1
	;

retStat : RETURN.0 SEMI.0.0
	| RETURN.0 exprList.0.0 SEMI.0.1
	| RETURN.0
	| RETURN.0 exprList.0.0
	;

elseIfBlock : ELSEIF.0 expr.0.0 LBRACE.0.1 chunk.0.2 RBRACE.0.3
	| ELSEIF.0 expr.0.0 LBRACE.0.1 RBRACE.0.2
	| ELSEIF.0 expr.0.0 LBRACE.0.1 RBRACE.0.2 elseIfBlock.0.3
	| ELSEIF.0 expr.0.0 LBRACE.0.1 chunk.0.2 RBRACE.0.3 elseIfBlock.0.4
	| ELSE.0 LBRACE.0.0 RBRACE.0.1
	| ELSE.0 LBRACE.0.0 chunk.0.1 RBRACE.0.2
	;

exprThen : expr.0 LBRACE.1 chunk.2
	| expr.0 LBRACE.1
	;

name : NAME
	;

nameList : NAME
	| nameList.0.0 COMMA.0 name.0.1
	;

exprList	: expr
	| exprList.0.0 COMMA.0 expr.0.1
	;

expr : logicalOrExp
	;

logicalOrExp : logicalAndExp
	| logicalOrExp.0.1 OR.0.0 logicalAndExp.0.0
	;

logicalAndExp : relationalExp
	| logicalAndExp.0.1 AND.0 relationalExp.0.0
	;

relationalExp : concatExp
	| relationalExp.0.1 LT.0 concatExp.0.0
	| relationalExp.0.1 GT.0 concatExp.0.0
	| relationalExp.0.1 LTEQ.0 concatExp.0.0
	| relationalExp.0.1 GTEQ.0 concatExp.0.0
	| relationalExp.0.1 NEQ.0 concatExp.0.0
	| relationalExp.0.1 EQ2.0 concatExp.0.0
	;

concatExp : additiveExp
	| additiveExp DOT2 concatExp
	;

additiveExp : multiplicativeExp
	| additiveExp.0.0 PLUS.0 multiplicativeExp.0.1
	| additiveExp.0.0 MINUS.0 multiplicativeExp.0.1
	;

multiplicativeExp : unaryExp
	| multiplicativeExp.0.0 ASTERISK.0 unaryExp.0.1
	| multiplicativeExp.0.0 DIVIDE.0 unaryExp.0.1
	| multiplicativeExp.0.0 PERCENT.0 unaryExp.0.1
	;

unaryExp : caretExp
	| NOT.0 unaryExp.0.0
	| SHARP.0 unaryExp.0.0
	| UMINUS.0 unaryExp.0.0
	;

caretExp : baseExp
	| baseExp.0.0 CARET.0 caretExp.0.1
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
	| LPAREN.0 expr.1 RPAREN.2
	;

fieldList : fieldListBody
	| fieldListBody COMMA
	| fieldListBody SEMIFIELD
	;

fieldListBody : field
	| fieldListBody COMMA field
	| fieldListBody SEMIFIELD field
	;

field : name EQ expr
	| expr
	;

var : NAME
	| prefixExp.0 DOT.0.0 NAME.0.1
	;

varList : var
	| varList COMMA var
	;

funcName : nameDotList
	| nameDotList COLON name
	;

nameDotList : NAME
	| nameDotList.0 DOT.0.0 NAME.0.1
	;
