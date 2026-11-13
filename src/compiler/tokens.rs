use super::diagnostics::Position;
use std::collections::HashSet;
use std::sync::LazyLock;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Identifier,
    Integer,
    Decimal,
    StringLiteral,
    StringObject,
    StringTemplate,
    String,
    Keyword,
    Operator,
    Dot,
    Comma,
    Colon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    At,
    Arrow,
    Hash,
    Tilde,
    EOF,

    // ABAP 2-character Comparison Operators
    CP,
    NP,
    CS,
    NS,
    CA,
    NA,
    CO,
    CN,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE,
}

impl TokenKind {
    pub fn is_comparison_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::CP
                | TokenKind::NP
                | TokenKind::CS
                | TokenKind::NS
                | TokenKind::CA
                | TokenKind::NA
                | TokenKind::CO
                | TokenKind::CN
                | TokenKind::EQ
                | TokenKind::NE
                | TokenKind::LT
                | TokenKind::GT
                | TokenKind::LE
                | TokenKind::GE
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub upper: String,
    pub start: Position,
    pub end: Position,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, start: Position, end: Position) -> Self {
        let upper = lexeme.to_ascii_uppercase();
        Self {
            kind,
            lexeme,
            upper,
            start,
            end,
        }
    }

    pub fn eof(pos: Position) -> Self {
        Self {
            kind: TokenKind::EOF,
            lexeme: String::new(),
            upper: String::new(),
            start: pos,
            end: pos,
        }
    }
}

pub static KEYWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let list = [
        "REPORT","PROGRAM","INITIALIZATION","AT","OUTPUT","ON","BLOCK","MEMORY","MODIF","MODE","CHARACTER","BYTE",
        "DATA","PARAMETERS","SELECT-OPTIONS","NO-INTERVALS","NO-EXTENSION","NO-GAPS","SELECTION-SCREEN","START-OF-SELECTION",
        "END-OF-SELECTION","VALUE-REQUEST","HELP-REQUEST","USER-COMMAND","MATCHCODE","RADIOBUTTON","GROUP","LOWER","CASE",
        "NO-DISPLAY","OBLIGATORY","AS","CHECKBOX","STATICS","CONSTANTS","TYPE","TYPES","VALUE","LIKE","REF","TO","OF",
        "TABLE","SORTED","HASHED","STANDARD","WITH","EMPTY","KEY","UNIQUE",
        "BEGIN","END","IF","ELSEIF","ELSE","ENDIF","WHILE","ENDWHILE","DO","TIMES","ENDDO","WHEN","OTHERS","ENDCASE",
        "LOOP","ENDLOOP","INTO","ASSIGNING","FIELD-SYMBOL","FIELD-SYMBOLS","ASSIGN","ASSIGNED",
        "APPEND","INSERT","READ","TRANSLATE","UPPER","CONCATENATE","SEPARATED","RESPECTING","BLANKS","SPLIT","CONDENSE",
        "USING","OPTIONS","FOR","INDEX","DELETE","SORT","ASCENDING","DESCENDING","DESCRIBE","LINES","CLEAR","CHECK",
        "CONTINUE","EXIT","RETURN","ASSERT","WRITE","SKIP",
        "CALL","FUNCTION","METHOD","ENDMETHOD","CLASS","ENDCLASS","INTERFACE","ENDINTERFACE","PUBLIC","PROTECTED","PRIVATE",
        "SECTION","IMPLEMENTATION","CREATE","OBJECT",
        "TRY","CATCH","ENDTRY","RAISE","EXCEPTION","NEW","THROW","SELECT","SINGLE","DISTINCT","FROM","FIELDS","WHERE",
        "ORDER","BY","UP","DOWN","HAVING","MAPPING",
        "AND","OR","NOT","IS","INITIAL","NULL","TRUE","FALSE","IN","BETWEEN","MOD","DIV","CONV","COND","SWITCH",
        "REDUCE","CORRESPONDING","EXACT","CAST",
        "IMPORTING","EXPORTING","CHANGING","ENTITIES","MODIFY","COMMIT","ROLLBACK","RESULT","FAILED","REPORTED","MAPPED",
        "ALL","UPDATE","RETURNING","RAISING","OPTIONAL","DEFAULT","FINAL","READ-ONLY","ABSTRACT","ALIASES","LOCAL",
        "FRIENDS","REDEFINITION","STATIC","INHERITING","SUPER","ME","CLASS-DATA","CLASS-METHODS","CLASS_CONSTRUCTOR",
        "REFERENCE","TYPE-POOLS",
        "INTEGER","INT8","INT4","INT2","STRING","XSTRING","CHAR","NUMC","DATS","TIMS","DECFLOAT16","DECFLOAT34","PACKED",
        "P","C","N","D","T","ABAP_BOOL","BOOLEAN",
        "ABAP_TRUE","ABAP_FALSE","SPACE","NEW-LINE","SY","SUBMATCHES","SUBSTRING","TO_UPPER","TO_LOWER","STRLEN",
    ];
    let mut set = HashSet::with_capacity(list.len());
    for k in list {
        set.insert(k);
    }
    set
});
