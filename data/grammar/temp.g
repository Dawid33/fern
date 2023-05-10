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

