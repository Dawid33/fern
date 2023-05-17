%nonterminal chunk
%nonterminal block
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
%terminal XEQ

%%


chunk : block
	| ENDFILE
	;

block : statList
	| retStat
	| statList RETURN SEMI
	| statList RETURN exprList SEMI
	| statList RETURN
	| statList RETURN exprList
	;

statList : stat
	| SEMI.0
	| stat.0.0 SEMI.0
	| statList.0.0 SEMI.0 stat.0.1
	| statList.0.0 SEMI.0
	;


stat :  varList XEQ exprList
	| functionCall
	| LBRACE block RBRACE
	| LBRACE RBRACE
	| WHILE expr LBRACE block RBRACE
	| WHILE expr LBRACE END
	| IF.0 exprThen.0.0 RBRACE.0.02
	| IF.0 exprThen.0.0 ELSE.0.1 block.0.1.0 RBRACE.0.2
	| IF.0 exprThen.0.0 ELSE.0.1 RBRACE.0.2
	| IF.0 exprThenElseIfB.0.0 RBRACE.0.1
	| IF.0 exprThenElseIfB.0.0 ELSE.0.1 block.0.1.0 RBRACE.0.2
	| IF.0 exprThenElseIfB.0.0 ELSE.0.1 RBRACE.0.2
	| FUNCTION funcName LBRACK parList RBRACK block RBRACE
	| FUNCTION funcName LBRACK RBRACK block RBRACE
	| FOR nameList IN exprList LBRACE block RBRACE
	| FOR nameList IN exprList LBRACE RBRACE
	| LET FUNCTION name LBRACK parList RBRACK block RBRACE
	| LET FUNCTION name LBRACK RBRACK block RBRACE
	| LET FUNCTION name LBRACK parList RBRACK RBRACE
	| LET FUNCTION name LBRACK RBRACK RBRACE
	| LET.0 nameList.0.0
	| LET.0 nameList.0.0.0 XEQ.0.0 exprList.0.0.1
	;

elseIfBlock : block ELSEIF expr LBRACE block
	| block ELSEIF expr LBRACE elseIfBlock
	| ELSEIF expr LBRACE block
	| block ELSEIF expr LBRACE
	| ELSEIF expr LBRACE
	| ELSEIF expr LBRACE elseIfBlock
	;

exprThenElseIfB : expr LBRACE elseIfBlock
 	;

exprThen : expr.0.0 LBRACE.0 block.0.1
	| expr.0 LBRACE.1
	;

name : NAME
	;

nameList : NAME
	| nameList.0.0 COMMA.0 name.0.1
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
	| relationalExp.0.1 LT.0 concatExp.0.0
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
	| additiveExp.0.0 PLUS.0 multiplicativeExp.0.1
	| additiveExp MINUS multiplicativeExp
	;

multiplicativeExp : unaryExp
	| multiplicativeExp.0.0 ASTERISK.0 unaryExp.0.1
	| multiplicativeExp.0.0 DIVIDE.0 unaryExp.0.1
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

functionCall : prefixExp LBRACK exprList RBRACK
	| prefixExp LBRACK RBRACK
	| prefixExp LBRACK fieldList RBRACK
	| prefixExp LBRACK RBRACK
	;

functionDef : FUNCTION LBRACK parList RBRACK block END
	| FUNCTION LBRACK RBRACK block END
	| FUNCTION LBRACK parList RBRACK END
	| FUNCTION LBRACK RBRACK END
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
