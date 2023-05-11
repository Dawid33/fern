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
	| SEMI
	| stat SEMI
	| statList SEMI stat
	| statList SEMI
	;


stat :  varList XEQ exprList
	| functionCall
	| LBRACE block RBRACE
	| LBRACE RBRACE
	| IF exprThen RBRACE
	| IF exprThen ELSE block RBRACE
	| IF exprThen ELSE RBRACE
	| IF exprThenElseIfB RBRACE
	| IF exprThenElseIfB ELSE block RBRACE
	| IF exprThenElseIfB ELSE RBRACE
	| LET nameList
	| LET nameList XEQ exprList
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

exprThen : expr LBRACE block
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

---------------

eCe : expr COMMA expr
	;

eCeCe  : eCe COMMA expr
	;

dot3 : DOT3
	;

retStat : RETURN SEMI
	| RETURN exprList SEMI
	| RETURN
	| RETURN exprList
	;

label : COLON2 NAME COLON2
	;

funcName : nameDotList
	| nameDotList COLON name
	;

nameDotList : NAME
	| nameDotList DOT NAME
	;

varList : var
	| varList COMMA var
	;

var : NAME
	| prefixExp LBRACK expr RBRACK
	| prefixExp DOT NAME
	;

nameList : NAME
	| nameList COMMA name
	;

exprList	: expr
	| exprList COMMA expr
	;

functionDef : FUNCTION LPARENFUNC parList RPARENFUNC block END
	| FUNCTION LPARENFUNC RPARENFUNC block END
	| FUNCTION LPARENFUNC parList RPARENFUNC END
	| FUNCTION LPARENFUNC RPARENFUNC END
	;

parList : nameList
	| nameList COMMA dot3
	| DOT3
	;

tableConstructor : LBRACE fieldList RBRACE
	| LBRACE RBRACE
	;

